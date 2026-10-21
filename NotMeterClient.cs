using System.Text.Json;

namespace Aion2Calculator;

// Public character endpoints observed on NotMeter. Only user-initiated lookups use this client.
internal sealed class NotMeterClient : IDisposable
{
    private readonly HttpClient http = new(new HttpClientHandler { AllowAutoRedirect = false })
    {
        BaseAddress = new Uri("https://notmeter.59-27-108-81.sslip.io/character/v1/"),
        Timeout = TimeSpan.FromSeconds(45)
    };

    public async Task<JsonElement> SearchAsync(string name, string region, string globalRegion)
    {
        if (string.IsNullOrWhiteSpace(name) || name.Length > 20) throw new ArgumentException("Enter a character name (up to 20 characters).");
        var regionQuery = RegionQuery(region, globalRegion);
        var url = $"search?name={Uri.EscapeDataString(name.Trim())}{regionQuery}&fast=1";
        JsonElement data = default;
        for (var attempt = 0; attempt < 8; attempt++)
        {
            data = await GetAsync(url);
            if (!data.TryGetProperty("complete", out var complete) || complete.ValueKind != JsonValueKind.False) break;
            await Task.Delay(1500);
        }
        return data;
    }

    public async Task<JsonElement> ProfileAsync(int serverId, string characterId, string region, string globalRegion)
    {
        if (serverId <= 0 || string.IsNullOrWhiteSpace(characterId) || characterId.Length > 128)
            throw new ArgumentException("Select a valid search result.");
        var url = $"profile?serverId={serverId}&characterId={Uri.EscapeDataString(characterId)}{RegionQuery(region, globalRegion)}&fast=1";
        JsonElement data = default;
        for (var attempt = 0; attempt < 8; attempt++)
        {
            data = await GetAsync(url);
            if (!data.TryGetProperty("complete", out var complete) || complete.ValueKind != JsonValueKind.False) break;
            await Task.Delay(1200);
        }
        return data;
    }

    private static string RegionQuery(string region, string globalRegion)
    {
        if (region is not ("kr" or "tw" or "ww")) throw new ArgumentException("Invalid region.");
        if (region == "ww" && globalRegion is not ("as" or "naw" or "nae" or "eu" or "la"))
            throw new ArgumentException("Select a Global region.");
        var language = region == "kr" ? "ko-KR" : region == "tw" ? "zh-TW" : "en-US";
        return $"&region={region}&lang={language}" + (region == "ww" ? $"&globalRegion={globalRegion}" : "");
    }

    private async Task<JsonElement> GetAsync(string path)
    {
        using var response = await http.GetAsync(path, HttpCompletionOption.ResponseHeadersRead);
        response.EnsureSuccessStatusCode();
        await using var stream = await response.Content.ReadAsStreamAsync();
        using var document = await JsonDocument.ParseAsync(stream);
        return document.RootElement.Clone();
    }

    public void Dispose() => http.Dispose();
}
