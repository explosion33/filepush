let register = function(username, password) {
    let xhr = new XMLHttpRequest();
    xhr.open("POST", "/register");

    xhr.setRequestHeader("Accept", "application/json");
    xhr.setRequestHeader("Content-Type", "application/json");

    xhr.onload = function() {
        if (xhr.status == 202) {
            window.location.href = "/";
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

let match_passwords = function() {
    let pass1 = document.getElementById("pass").value;
    let pass2 = document.getElementById("pass2").value;

    if (pass1 != pass2 || pass1=="") {
        document.getElementById("match").style.visibility = "visible";
        document.getElementById("regbtn").disabled = true;
    }
    else {
        document.getElementById("match").style.visibility = "hidden";
        document.getElementById("regbtn").disabled = false;
    }
}

let validate_username = function() {
    let name = document.getElementById("user").value;
    let pass1 = document.getElementById("pass").value;
    let pass2 = document.getElementById("pass2").value;

    if (name == "" || pass1 == "" || pass2 == "") {
        document.getElementById("regbtn").disabled = true;
    }
    else {
        document.getElementById("regbtn").disabled = false;
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