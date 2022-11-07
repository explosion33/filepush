let login = async function(username, password) {
    sessionStorage.setItem("username", username);
    sessionStorage.setItem("password", password);
    window.location.href = "/view";
}

let verify_user = function(username, password) {
    let xhr = new XMLHttpRequest();
    xhr.open("POST", "/user");

    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Content-Type", "application/json");

    xhr.onload = function() {
        if (xhr.status == 202) {
            login(username, password)
        }
        display_error(xhr.responseText);
    };

    let data = {
        "username": username,
        "password": password 
    };

    xhr.send(JSON.stringify(data));
}

let display_error = function(error) {
    document.getElementById("error").innerText = error;
}

window.onload = function() {
    let username = sessionStorage.getItem("username");
    let password = sessionStorage.getItem("password");

    if (username == null || password == null || username == "" || password == "") {
        console.log("no user found");
    }
    else {
        window.location.href = "/view";
    }
}

document.addEventListener("keypress", function(event) {
    // If the user presses the "Enter" key on the keyboard
    if (event.key === "Enter") {
      // Cancel the default action, if needed
      event.preventDefault();
      // Trigger the button element with a click
      document.getElementById("loginbtn").click();
    }
  }); 