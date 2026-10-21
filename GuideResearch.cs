using System.Net;
using System.Text.Json;
using System.Text.RegularExpressions;

namespace Aion2Calculator;

internal sealed class GuideResearch : IDisposable
{
    private static readonly IReadOnlyDictionary<string, string> Guides = new Dictionary<string, string>
    {
        ["attack"] = "https://aion2maps.com/guides/stats-explained/",
        ["pve"] = "https://aion2maps.com/guides/how-damage-works/",
        ["pvp"] = "https://aion2maps.com/guides/how-damage-works/",
        ["chance"] = "https://aion2maps.com/guides/accuracy-and-crit/",
        ["creature"] = "https://aion2maps.com/guides/stats-explained/"
    };
    private readonly HttpClient client = new() { Timeout = TimeSpan.FromSeconds(12) };

    public async Task<JsonElement> FetchAsync(string topic)
    {
        if (!Guides.TryGetValue(topic, out var url)) throw new ArgumentException("Unknown guide topic.");
        using var request = new HttpRequestMessage(HttpMethod.Get, url);
        using var response = await client.SendAsync(request);
        response.EnsureSuccessStatusCode();
        var text = await response.Content.ReadAsStringAsync();
        var articleStart = text.IndexOf("<article", StringComparison.OrdinalIgnoreCase);
        if (articleStart < 0) throw new InvalidOperationException("Guide article was not found.");
        var articleEnd = text.IndexOf("</article>", articleStart, StringComparison.OrdinalIgnoreCase);
        if (articleEnd < 0) throw new InvalidOperationException("Guide article is incomplete.");
        text = text.Substring(articleStart, articleEnd - articleStart);
        text = Regex.Replace(text, "<script\\b[^>]*>.*?</script>|<style\\b[^>]*>.*?</style>", " ", RegexOptions.IgnoreCase | RegexOptions.Singleline);
        text = WebUtility.HtmlDecode(Regex.Replace(text, "<[^>]+>", " "));
        text = Regex.Replace(text, "\\s+", " ").Trim();
        return JsonSerializer.SerializeToElement(new { url, excerpt = text[..Math.Min(3000, text.Length)] });
    }

    public void Dispose() => client.Dispose();
}
