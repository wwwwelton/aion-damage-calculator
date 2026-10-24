using System.Net;
using System.Text.Json;
using System.Text.Json.Nodes;

namespace Aion2Calculator;

// Public character lookups from the official AION 2 site (aion2.plaync.com), using the same
// search and character APIs as its /en-us/characters pages. Only user-initiated lookups use this client.
internal sealed class OfficialCharacterClient : IDisposable
{
    private const string SearchEndpoint = "https://api-search.plaync.com/aion2global/search/v2/character";
    private const string CharacterEndpoint = "https://aion2.plaync.com/api/character/";
    // Imported stat labels are matched in English, so the official data is always requested in en-US.
    private const string Language = "en-US";
    private static readonly string[] GlobalRegions = ["as", "naw", "nae", "eu", "la"];
    private static readonly SemaphoreSlim ItemRequests = new(6);

    private readonly HttpClient http = new(new HttpClientHandler
    {
        AutomaticDecompression = DecompressionMethods.All
    })
    {
        // Fail fast: a healthy official response arrives in a few seconds.
        Timeout = TimeSpan.FromSeconds(20)
    };
    private readonly Dictionary<string, CacheEntry> cache = [];
    private readonly object cacheLock = new();
    private readonly record struct CacheEntry(DateTimeOffset ExpiresAt, JsonElement Data);
    private const int MaxCacheEntries = 128;

    public OfficialCharacterClient()
    {
        http.DefaultRequestHeaders.UserAgent.ParseAdd("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AION2-Calculator");
        http.DefaultRequestHeaders.Accept.ParseAdd("application/json");
        http.DefaultRequestHeaders.Referrer = new Uri("https://aion2.plaync.com/en-us/characters/index");
    }

    public async Task<JsonElement> SearchAsync(string name, string region, string globalRegion)
    {
        if (string.IsNullOrWhiteSpace(name) || name.Length > 20) throw new ArgumentException("Enter a character name (up to 20 characters).");
        RequireGlobalRegion(region, globalRegion);
        var url = $"{SearchEndpoint}?keyword={Uri.EscapeDataString(name.Trim())}&region={globalRegion}&localeInfo={Language}&size=40";
        if (TryGetCached(url, out var cached)) return cached;
        var response = await GetJsonAsync(url);
        var results = new JsonArray();
        if (response.TryGetProperty("list", out var list) && list.ValueKind == JsonValueKind.Array)
        {
            foreach (var entry in list.EnumerateArray())
            {
                results.Add(new JsonObject
                {
                    ["name"] = StripMarkup(entry.GetStringOrEmpty("name")),
                    ["characterId"] = entry.GetStringOrEmpty("characterId"),
                    ["serverId"] = entry.TryGetProperty("serverId", out var serverId) ? serverId.GetInt32() : 0,
                    ["serverName"] = entry.GetStringOrEmpty("serverName"),
                    ["level"] = entry.TryGetProperty("level", out var level) ? level.GetInt32() : 0,
                    ["region"] = globalRegion
                });
            }
        }
        var data = JsonSerializer.SerializeToElement(new JsonObject { ["results"] = results, ["complete"] = true });
        StoreComplete(url, data, TimeSpan.FromMinutes(2));
        return data;
    }

    public async Task<JsonElement> ProfileAsync(int serverId, string characterId, string region, string globalRegion)
    {
        if (serverId <= 0 || string.IsNullOrWhiteSpace(characterId) || characterId.Length > 128)
            throw new ArgumentException("Select a valid search result.");
        RequireGlobalRegion(region, globalRegion);
        // Search results already carry the encoded ID; decode once so it is encoded exactly once below.
        var id = Uri.UnescapeDataString(characterId);
        var query = $"region={globalRegion}&lang={Language}&characterId={Uri.EscapeDataString(id)}&serverId={serverId}";
        var url = $"profile?{query}";
        if (TryGetCached(url, out var cached)) return cached;

        var infoTask = GetJsonAsync($"{CharacterEndpoint}info?{query}");
        var equipmentTask = GetJsonAsync($"{CharacterEndpoint}equipment?{query}");
        await Task.WhenAll(infoTask, equipmentTask);
        var equipment = equipmentTask.Result;

        var details = new JsonObject();
        var complete = true;
        var items = equipment.TryGetProperty("equipment", out var equipmentSection)
            && equipmentSection.TryGetProperty("equipmentList", out var equipmentList)
            ? equipmentList.EnumerateArray().ToList() : [];
        var detailTasks = items.Select(item => ItemDetailAsync(item, query)).ToArray();
        await Task.WhenAll(detailTasks);
        for (var index = 0; index < items.Count; index++)
        {
            var detail = detailTasks[index].Result;
            if (detail is null) { complete = false; continue; }
            details[items[index].GetProperty("slotPos").GetInt32().ToString()] = detail;
        }

        var data = JsonSerializer.SerializeToElement(new JsonObject
        {
            ["info"] = JsonNode.Parse(infoTask.Result.GetRawText()),
            ["equipment"] = JsonNode.Parse(equipment.GetRawText()),
            ["itemDetails"] = details,
            ["complete"] = complete
        });
        StoreComplete(url, data, TimeSpan.FromMinutes(5));
        return data;
    }

    private async Task<JsonNode?> ItemDetailAsync(JsonElement item, string query)
    {
        try
        {
            await ItemRequests.WaitAsync();
            try
            {
                var url = $"{CharacterEndpoint}equipment/item?{query}&id={item.GetProperty("id").GetInt64()}"
                    + $"&enchantLevel={item.GetProperty("enchantLevel").GetInt32()}&slotPos={item.GetProperty("slotPos").GetInt32()}";
                var detail = await GetJsonAsync(url);
                return JsonNode.Parse(detail.GetRawText());
            }
            finally
            {
                ItemRequests.Release();
            }
        }
        catch (Exception error) when (error is HttpRequestException or TaskCanceledException or JsonException or KeyNotFoundException or InvalidOperationException)
        {
            return null;
        }
    }

    private static void RequireGlobalRegion(string region, string globalRegion)
    {
        if (region != "ww" || !GlobalRegions.Contains(globalRegion))
            throw new ArgumentException("Official lookup is available for Global regions only. Select a Global region.");
    }

    private static string StripMarkup(string text) => System.Text.RegularExpressions.Regex.Replace(text, "<[^>]+>", "");

    private bool TryGetCached(string key, out JsonElement data)
    {
        lock (cacheLock)
        {
            if (cache.TryGetValue(key, out var entry) && entry.ExpiresAt > DateTimeOffset.UtcNow)
            {
                data = entry.Data;
                return true;
            }
            cache.Remove(key);
        }
        data = default;
        return false;
    }

    private void StoreComplete(string key, JsonElement data, TimeSpan lifetime)
    {
        if (data.TryGetProperty("complete", out var complete) && complete.ValueKind == JsonValueKind.False) return;
        lock (cacheLock)
        {
            if (cache.Count >= MaxCacheEntries && !cache.ContainsKey(key))
            {
                string? evictionKey = null;
                var earliestExpiry = DateTimeOffset.MaxValue;
                foreach (var pair in cache)
                {
                    if (pair.Value.ExpiresAt >= earliestExpiry) continue;
                    earliestExpiry = pair.Value.ExpiresAt;
                    evictionKey = pair.Key;
                }
                if (evictionKey is not null) cache.Remove(evictionKey);
            }
            cache[key] = new CacheEntry(DateTimeOffset.UtcNow.Add(lifetime), data);
        }
    }

    private async Task<JsonElement> GetJsonAsync(string url)
    {
        using var response = await http.GetAsync(url, HttpCompletionOption.ResponseHeadersRead);
        response.EnsureSuccessStatusCode();
        await using var stream = await response.Content.ReadAsStreamAsync();
        using var document = await JsonDocument.ParseAsync(stream);
        return document.RootElement.Clone();
    }

    public void Dispose() => http.Dispose();
}

internal static class JsonElementExtensions
{
    public static string GetStringOrEmpty(this JsonElement element, string property) =>
        element.TryGetProperty(property, out var value) && value.ValueKind == JsonValueKind.String ? value.GetString() ?? "" : "";
}
