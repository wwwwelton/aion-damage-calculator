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
        ["calculator.js"] = "text/javascript; charset=utf-8",
        ["numeric-input.js"] = "text/javascript; charset=utf-8",
        ["recommendations.js"] = "text/javascript; charset=utf-8",
        ["matchups.js"] = "text/javascript; charset=utf-8",
        ["renderer.js"] = "text/javascript; charset=utf-8",
        ["characters.js"] = "text/javascript; charset=utf-8",
        ["boss-data.js"] = "text/javascript; charset=utf-8",
        ["bosses.js"] = "text/javascript; charset=utf-8"
    };

    private readonly WebView2 view = new() { Dock = DockStyle.Fill };
    private readonly NotMeterClient notMeter = new();

    public CalculatorWindow()
    {
        Text = "AION 2 — Damage Calculator";
        Width = 1100;
        Height = 820;
        MinimumSize = new Size(760, 600);
        Controls.Add(view);
        Shown += async (_, _) => await InitializeViewAsync();
        FormClosed += (_, _) => notMeter.Dispose();
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
            var region = request.GetProperty("region").GetString() ?? "";
            var globalRegion = request.GetProperty("globalRegion").GetString() ?? "";
            JsonElement data = kind switch
            {
                "search" => await notMeter.SearchAsync(request.GetProperty("name").GetString() ?? "", region, globalRegion),
                "profile" => await notMeter.ProfileAsync(request.GetProperty("serverId").GetInt32(),
                    request.GetProperty("characterId").GetString() ?? "", region, globalRegion),
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
        if (ContentTypes.TryGetValue(name, out var type))
        {
            var stream = Assembly.GetExecutingAssembly().GetManifestResourceStream(name);
            if (stream is not null)
            {
                e.Response = view.CoreWebView2.Environment.CreateWebResourceResponse(
                    stream, 200, "OK", $"Content-Type: {type}\r\nCache-Control: no-store");
                return;
            }
        }
        e.Response = view.CoreWebView2.Environment.CreateWebResourceResponse(
            null, 404, "Not Found", "Content-Type: text/plain");
    }
}
