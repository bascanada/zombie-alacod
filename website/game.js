document.addEventListener('DOMContentLoaded', function() {
    const urlParams = new URLSearchParams(window.location.search);
    const online = urlParams.get("online");
    const lobby = urlParams.get("lobby");
    const name = "./" + urlParams.get("name") + "/wasm.js";

    if (online === "true") {
      let canvas = document.getElementById("bevy-canvas");
      canvas.setAttribute("data-matchbox", "wss://matchbox.bascanada.org");
      canvas.setAttribute("data-number-player", "2");
    }

    import(name).then((module) => {
      console.log(module);
      module.default();
      auto_focus();
    });

    function auto_focus() {
      let canvas = document.getElementsByTagName("bevy-canvas");

      if (!lobby) {
        alert("You failed to provide a lobby , reload the page with ?lobby=mylobbyname as arguments after the path");
      }

      if (!canvas.length) {
        setTimeout(auto_focus, 100);
      } else {
        canvas[0].focus();
      }
    }
});