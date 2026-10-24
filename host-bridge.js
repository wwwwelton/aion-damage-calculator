// Injected before the page scripts by the native host. Pages talk to the host only through
// aion2Host: post() sends a request object, onMessage() receives replies as { data }.
(() => {
  const listeners = [];
  window.aion2Host = {
    post(message) {
      window.ipc.postMessage(JSON.stringify(message));
    },
    onMessage(listener) {
      listeners.push(listener);
    },
    receive(data) {
      for (const listener of listeners) listener({ data });
    }
  };
})();
