using System.Reflection;
using System.Text.Json;
using Microsoft.Web.WebView2.Core;
using Microsoft.Web.WebView2.WinForms;

namespace Aion2Calculator;

internal static class Program
{
    [STAThread]
    private static void Main()
    {
        ApplicationConfiguration.Initialize();
        Application.Run(new CalculatorWindow());
    }
}

internal sealed class CalculatorWindow : Form
{
    private static readonly Uri Home = new("https://aion2.local/index.html");
    private static readonly Dictionary<string, string> ContentTypes = new()
    {
        ["index.html"] = "text/html; charset=utf-8",
        ["style.css"] = "text/css; charset=utf-8",
        ["i18n.js"] = "text/javascript; charset=utf-8",
        ["calculator.js"] = "text/javascript; charset=utf-8",
        ["numeric-input.js"] = "text/javascript; charset=utf-8",
        ["recommendations.js"] = "text/javascript; charset=utf-8",
        ["icons.js"] = "text/javascript; charset=utf-8",
        ["matchups.js"] = "text/javascript; charset=utf-8",
        ["ai-tips.js"] = "text/javascript; charset=utf-8",
        ["ai-worker.bundle.js"] = "text/javascript; charset=utf-8",
        ["assets/ai/ort-wasm-simd-threaded.jsep.mjs"] = "text/javascript; charset=utf-8",
        ["assets/ai/ort-wasm-simd-threaded.jsep.wasm"] = "application/wasm",
        ["assets/ai/models/onnx-community/Qwen3-0.6B-ONNX/config.json"] = "application/json",
        ["assets/ai/models/onnx-community/Qwen3-0.6B-ONNX/tokenizer_config.json"] = "application/json",
        ["assets/ai/models/onnx-community/Qwen3-0.6B-ONNX/tokenizer.json"] = "application/json",
        ["assets/ai/models/onnx-community/Qwen3-0.6B-ONNX/generation_config.json"] = "application/json",
        ["assets/ai/models/onnx-community/Qwen3-0.6B-ONNX/onnx/model_q4f16.onnx"] = "application/octet-stream",
        ["profile-ui.js"] = "text/javascript; charset=utf-8",
        ["stat-tooltips.js"] = "text/javascript; charset=utf-8",
        ["renderer.js"] = "text/javascript; charset=utf-8",
        ["characters.js"] = "text/javascript; charset=utf-8",
        ["boss-data.js"] = "text/javascript; charset=utf-8",
        ["mob-data.js"] = "text/javascript; charset=utf-8",
        ["bosses.js"] = "text/javascript; charset=utf-8"
    };

    private readonly WebView2 view = new() { Dock = DockStyle.Fill };
    private readonly NotMeterClient notMeter = new();
    private readonly GuideResearch guides = new();

    public CalculatorWindow()
    {
        Text = "AION 2 — Damage Calculator";
        Width = 1100;
        Height = 820;
        MinimumSize = new Size(760, 600);
        Controls.Add(view);
        Shown += async (_, _) => await InitializeViewAsync();
        FormClosed += (_, _) => { notMeter.Dispose(); guides.Dispose(); };
    }

    private async Task InitializeViewAsync()
    {
        try
        {
            var dataFolder = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "AION2-Calculadora", "WebView2");
            var environment = await CoreWebView2Environment.CreateAsync(userDataFolder: dataFolder);
            await view.EnsureCoreWebView2Async(environment);
            view.CoreWebView2.Settings.AreDevToolsEnabled = false;
            view.CoreWebView2.Settings.IsWebMessageEnabled = true;
            view.CoreWebView2.AddWebResourceRequestedFilter("https://aion2.local/*", CoreWebView2WebResourceContext.All);
            view.CoreWebView2.WebResourceRequested += ServeEmbeddedResource;
            view.CoreWebView2.WebMessageReceived += OnWebMessage;
            view.CoreWebView2.NavigationStarting += (_, e) =>
            {
                if (!Uri.TryCreate(e.Uri, UriKind.Absolute, out var uri)
                    || uri.Scheme != Uri.UriSchemeHttps || uri.Host != "aion2.local")
                    e.Cancel = true;
            };
            view.CoreWebView2.Navigate(Home.AbsoluteUri);
        }
        catch (Exception error)
        {
            MessageBox.Show(this,
                $"Could not start WebView2. Install the Microsoft Edge WebView2 Runtime.\n\n{error.Message}",
                "Unable to open calculator", MessageBoxButtons.OK, MessageBoxIcon.Error);
            Close();
        }
    }

    private async void OnWebMessage(object? sender, CoreWebView2WebMessageReceivedEventArgs e)
    {
        if (!Uri.TryCreate(e.Source, UriKind.Absolute, out var source)
            || source.Scheme != Uri.UriSchemeHttps || source.Host != Home.Host) return;
        int id;
        try
        {
            using var message = JsonDocument.Parse(e.WebMessageAsJson);
            var request = message.RootElement;
            id = request.GetProperty("id").GetInt32();
            var kind = request.GetProperty("kind").GetString();
            var region = request.TryGetProperty("region", out var requestedRegion) ? requestedRegion.GetString() ?? "" : "";
            var globalRegion = request.TryGetProperty("globalRegion", out var requestedGlobal) ? requestedGlobal.GetString() ?? "" : "";
            JsonElement data = kind switch
            {
                "search" => await notMeter.SearchAsync(request.GetProperty("name").GetString() ?? "", region, globalRegion),
                "profile" => await notMeter.ProfileAsync(request.GetProperty("serverId").GetInt32(),
                    request.GetProperty("characterId").GetString() ?? "", region, globalRegion),
                "research" => await guides.FetchAsync(request.GetProperty("topic").GetString() ?? ""),
                _ => throw new ArgumentException("Unknown character request.")
            };
            if (!view.IsDisposed) view.CoreWebView2.PostWebMessageAsJson(JsonSerializer.Serialize(new { id, data }));
        }
        catch (Exception error)
        {
            // Keep malformed or failed requests visible to the user without exposing the native process.
            try
            {
                using var message = JsonDocument.Parse(e.WebMessageAsJson);
                id = message.RootElement.GetProperty("id").GetInt32();
                if (!view.IsDisposed)
                    view.CoreWebView2.PostWebMessageAsJson(JsonSerializer.Serialize(new { id, error = error.Message }));
            }
            catch (Exception) { }
        }
    }

    private void ServeEmbeddedResource(object? sender, CoreWebView2WebResourceRequestedEventArgs e)
    {
        var uri = new Uri(e.Request.Uri);
        var name = uri.AbsolutePath.TrimStart('/');
        if (name.StartsWith("assets/classes/", StringComparison.Ordinal)
            && name.EndsWith(".webp", StringComparison.Ordinal)
            && name.Count(c => c == '/') == 2)
        {
            var className = name["assets/classes/".Length..^".webp".Length];
            if (className is "gladiator" or "templar" or "assassin" or "ranger" or "sorcerer"
                or "spiritmaster" or "cleric" or "chanter")
            {
                var image = Assembly.GetExecutingAssembly().GetManifestResourceStream(name);
                if (image is not null)
                {
                    e.Response = view.CoreWebView2.Environment.CreateWebResourceResponse(
                        image, 200, "OK", "Content-Type: image/webp\r\nCache-Control: public, max-age=31536000, immutable");
                    return;
                }
            }
        }
        if (ContentTypes.TryGetValue(name, out var type))
        {
            var stream = Assembly.GetExecutingAssembly().GetManifestResourceStream(name);
            if (stream is not null)
            {
                var cacheControl = name.StartsWith("assets/ai/", StringComparison.Ordinal)
                    ? "public, max-age=31536000, immutable" : "no-store";
                e.Response = view.CoreWebView2.Environment.CreateWebResourceResponse(
                    stream, 200, "OK", $"Content-Type: {type}\r\nCache-Control: {cacheControl}");
                return;
            }
        }
        e.Response = view.CoreWebView2.Environment.CreateWebResourceResponse(
            null, 404, "Not Found", "Content-Type: text/plain");
    }
}
