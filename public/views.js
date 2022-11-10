var drop = function(event) {
	var event = window.event || event;
	event.preventDefault();
	
	let dt = event.dataTransfer
	let files = dt.files
	console.log(files);

	if (files.length == 0) {
		errorUploading("no file");
	}
	else if (files.length == 1) {
		uploadFile(files[0])
	}
	else {
		packFiles(files);
	}
}

var dialog = function(event) {
	var event = window.event || event;

	var files = event.target.files; 
	
	console.log(files);
	
	if (files.length == 0) {
		errorUploading(0);
	}
	else if (files.length == 1) {
		uploadFile(files[0])
	}
	else {
		packFiles(files);
	}
	
}

var allowDrop = function(event) {
	var event = window.event || event;
	event.preventDefault();
}

function errorUploading(error) {
	console.log("error " + error);

	let errorDiv = document.getElementById("error-corner");
	let html = "";

	html += `<div class="container">`
	html += `<div class="alert alert-danger alert-dismissible">`;
	html += `<a href="#" class="close" data-dismiss="alert" aria-label="close">&times;</a>`;
	html += "<strong>Error</strong> Could not upload file (" + error + ")";
	html += "</div></div>";

	errorDiv.innerHTML = html;
	
}

let show_login = function() {
    sessionStorage.removeItem("username");
    sessionStorage.removeItem("password");
    window.location.href = "/"
}

//sends a file to /storeFile
function uploadFile(file) {
	var xhr = new XMLHttpRequest()
	var formData = new FormData()
	xhr.open('POST', "/file_upload", true)
  
	xhr.addEventListener('readystatechange', function(e) {
		if (xhr.readyState == 4 && xhr.status == 202) {
			localStorage.setItem(xhr.responseText, file.name);

            let bar = document.getElementById("progress-bar");
            bar.style.width = "0%";
            bar.innerText = "COMPLETED";

            show_image(xhr.responseText, false);


		}
		else if (xhr.readyState == 4 && xhr.status != 202) {
			show_error(xhr.responseText);
		}
	});

	xhr.upload.addEventListener("progress", function(e) {
		if (e.lengthComputable) {
			var percentComplete = e.loaded / e.total;
			percentComplete = parseInt(percentComplete * 100);
		  
			let bar = document.getElementById("progress-bar");
			bar.style.width = percentComplete + "%";
			if (percentComplete >= 5) {
				bar.innerText = percentComplete + "%";
			}

		}
	  }, false);
  
  
      xhr.setRequestHeader("username", sessionStorage.getItem("username"));
      xhr.setRequestHeader("password", sessionStorage.getItem("password"));
      xhr.setRequestHeader("filename", file.name)
      xhr.setRequestHeader("bytes", file.size)
  
      console.log(file);
      xhr.send(file);
}


let copy = function(img) {
    navigator.clipboard.writeText(img.id);

    if (img.className != "clicked") {
        img.className = "clicked"
        //img.src = "/static/copy_green.png"

        setTimeout(function () {
            img.className = "copy_normal"
        }, 1000)
    }
}

let get_user_images = function(username, password) {
    let xhr = new XMLHttpRequest();
    xhr.open("GET", "/images");

    xhr.onload = function() {
        console.log(xhr.status);
        if (xhr.status == 200) {
            JSON.parse(xhr.responseText).forEach(pair => {
                show_image(pair[0], pair[1]);
            });
        }
        else {
            display_error(xhr.responseText);
        }
    };

    xhr.setRequestHeader("username", username);
    xhr.setRequestHeader("password", password);

    xhr.send();
}

let show_image = function(filename, public) {
    username = sessionStorage.getItem("username");
    let element =
    `
    <div class="row">
        <div class="col-sm-9 align-items-center">
            <p name="${window.location.origin}/${username}/${filename}">${filename}</p>
        </div>
        
        <div class="col-sm-2">
            <input name="${window.location.origin}/${username}/${filename}" type="checkbox" onchange="update_visibility(this)" ${public ? "checked" : ""}>
        </div>
        <div class="col-sm-1">
            <img class="copy_normal" id="${window.location.origin}/${username}/${filename}" name="${filename}" src="/static/copy.png" style="width: 20px;" onclick="copy(this);" ${public ? "" : "hidden"}>
        </div>
    </div>
    `

    document.getElementById("files").innerHTML += element;
}

let update_visibility = function(checkbox) {
    console.log(checkbox.checked);
    let img = document.getElementById(checkbox.name);
    img.hidden = !checkbox.checked;

    let xhr = new XMLHttpRequest();
    xhr.open("POST", "/permissions/" + img.name + "/" + checkbox.checked);

    xhr.onload = function() {
        console.log(xhr.status);
        if (xhr.status == 202) {
        }
        else {
            checkbox.checked = false;
        }
    };

    xhr.setRequestHeader("username", sessionStorage.getItem("username"));
    xhr.setRequestHeader("password", sessionStorage.getItem("password"));

    xhr.send();


}

window.onload = function() {
    let username = sessionStorage.getItem("username");
    let password = sessionStorage.getItem("password");

    if (username == null || password == null || username == "" || password == "") {
        show_login();
    }
    else {
        document.getElementById("title").innerText = username;
        document.getElementById("username").innerText = username;
        get_user_images(username, password);
    }
}