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

            // REDIRECT HERE

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

window.onload = function() {
    let username = sessionStorage.getItem("username");
    let password = sessionStorage.getItem("password");

    if (username == null || password == null || username == "" || password == "") {
        show_login();
    }
    else {
        document.getElementById("title").innerText = username;
        document.getElementById("username").innerText = username;
    }
}