// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

const prepSubdir = (link) => {
    let thisPage = new URL(window.location.href);
    let subdir = thisPage.pathname;
    let out = (subdir + link).replace('//', '/');
    console.log(out);
    return (subdir + link).replace('//', '/');
}

const getSiteUrl = async () => {
    let url = await fetch(prepSubdir("/api/siteurl"))
                .then(res => res.text());
    if (url == "unset") {
        return window.location.host.replace(/\/$/, '');
    }
    else {
        return url.replace(/\/$/, '').replace(/^"/, '').replace(/"$/, '');
    }
}

const getVersion = async () => {
    let ver = await fetch(prepSubdir("/api/version"))
                .then(res => res.text());
    return ver;
}

const showVersion = async () => {
    let version = await getVersion();
    link = document.getElementById("version-number");
    link.innerText = "v" + version;
    link.href = "https://github.com/minoplhy/chhoto-url/releases/tag/" + version;
    link.hidden = false;
}

const getLogin = async () => {
    document.getElementById("container").style.filter = "blur(2px)";
    document.getElementById("login-dialog").showModal();
    document.getElementById("password").focus();
};

document.addEventListener("DOMContentLoaded", () => {
    const adminButton = document.getElementById("admin-button");
    if (adminButton) {
        adminButton.addEventListener("click", (e) => {
            e.preventDefault();
            getLogin();
        });
    }
});

const refreshData = async () => {
    let res = await fetch(prepSubdir("/api/all"));
    if (!res.ok) {
        let errorMsg = await res.text();
        document.getElementById("url-table").innerHTML = '';
        console.log(errorMsg);
        if (errorMsg == "Using public mode.") {
            document.getElementById("admin-button").hidden = false;
            document.getElementById("api-key-button").hidden = true; // Hide initially
            document.getElementById("reset-api-key-button").hidden = true;
            loading_text = document.getElementById("loading-text");
            loading_text.hidden = true;
            showVersion();
        } else {
            getLogin();
        }
    } else {
        let data = await res.json();
        displayData(data.reverse());
        document.getElementById("api-key-button").hidden = false; // Show API Key button when logged in
        document.getElementById("reset-api-key-button").hidden = false;
    }
}

const displayData = async (data) => {
    showVersion();
    let site = await getSiteUrl();

    const admin_button = document.getElementById("admin-button");
    admin_button.innerText = "logout";
    admin_button.removeAttribute("href");
    admin_button.hidden = false;

    const newButton = admin_button.cloneNode(true);
    newButton.href = "#";
    admin_button.parentNode.replaceChild(newButton, admin_button);

    newButton.addEventListener("click", (e) => {
        e.preventDefault();
        logOut();
    });

    apikey_button = document.getElementById("api-key-button");
    apikey_button.innerText = "API Key"
    apikey_button.hidden = false;

    reset_apikey_button = document.getElementById("reset-api-key-button");
    reset_apikey_button.innerText = "Reset API Key"
    reset_apikey_button.hidden = false;

    table_box = document.getElementById("table-box");
    loading_text = document.getElementById("loading-text");
    const table = document.getElementById("url-table");

    if (data.length == 0) {
        table_box.hidden = true;
        loading_text.innerHTML = "No active links.";
        loading_text.hidden = false;
    }
    else {
        loading_text.hidden = true;
        if (!window.isSecureContext) {
            const shortUrlHeader = document.getElementById("short-url-header");
            shortUrlHeader.innerHTML = "Short URL<br>(right click and copy)";
        }
        table_box.hidden = false;
        table.innerHTML = '';
        data.forEach(tr => table.appendChild(TR(tr, site)));
    }
}

const showAlert = async (text, col) => {
    document.getElementById("alert-box")?.remove();
    const controls = document.getElementById("controls");
    const alertBox = document.createElement("p");
    alertBox.id = "alert-box";
    alertBox.style.color = col;
    alertBox.innerHTML = text;
    controls.appendChild(alertBox);
}

document.addEventListener("click", (e) => {
    const link = e.target.closest(".copy-short-url");
    if (link) {
        e.preventDefault();
        const shortlink = link.dataset.shortlink;
        copyShortUrl(shortlink);
    }
});

const TR = (row, site) => {
    const tr = document.createElement("tr");
    const longTD = TD(A_LONG(row["longlink"]), "Long URL");
    var shortTD = null;
    var isSafari = /Safari/.test(navigator.userAgent) && /Apple Computer/.test(navigator.vendor);
    // For now, we disable copying on WebKit due to a possible bug. Manual copying is enabled instead.
    // Take a look at https://github.com/SinTan1729/chhoto-url/issues/36
    if (window.isSecureContext && !(isSafari)) {
        shortTD = TD(A_SHORT(row["shortlink"], site), "Short URL");
    } else {
        shortTD = TD(A_SHORT_INSECURE(row["shortlink"], site), "Short URL");
    }
    let hitsTD = TD(row["hits"]);
    hitsTD.setAttribute("label", "Hits");
    hitsTD.setAttribute("name", "hitsColumn");
    const deleteBtn = deleteButton(row["shortlink"]);
    const editBtn = editButton(row["shortlink"]);

    tr.appendChild(shortTD);
    tr.appendChild(longTD);
    tr.appendChild(hitsTD);
    tr.appendChild(editBtn);
    tr.appendChild(deleteBtn);

    return tr;
}

const copyShortUrl = async (link) => {
    const site = await getSiteUrl();
    try {
        navigator.clipboard.writeText(`${site}/${link}`);
        showAlert(`Short URL ${link} was copied to clipboard!`, "green");
    } catch (e) {
        console.log(e);
        showAlert(`Could not copy short URL to clipboard, please do it manually: <a href=${site}/${link}>${site}/${link}</a>`, "red");
    }

}

const addProtocol = (input) => {
    var url = input.value.trim();
    if (url != "" && !~url.indexOf("://") && !~url.indexOf("magnet:")) {
        url = "https://" + url;
    }
    input.value = url;
    return input;
}

document.addEventListener("DOMContentLoaded", () => {
    const longUrlInput = document.getElementById("longUrl");

    longUrlInput.addEventListener("blur", () => {
        addProtocol(longUrlInput);
    });
});

const A_LONG = (s) => `<a href='${s}'>${s}</a>`;
const A_SHORT = (s, t) => `<a href="#" class="copy-short-url" data-shortlink="${s}">${s}</a>`;
const A_SHORT_INSECURE = (s, t) => `<a href="${t}/${s}">${s}</a>`;

const deleteButton = (shortUrl) => {
    const td = document.createElement("td");
    const div = document.createElement("div");
    const btn = document.createElement("button");

    btn.innerHTML = "&times;";

    btn.onclick = e => {
        e.preventDefault();
        if (confirm("Do you want to delete the entry " + shortUrl + "?")) {
            document.getElementById("alert-box")?.remove();
            showAlert("&nbsp;", "black");
            fetch(prepSubdir(`/api/del/${shortUrl}`), {
                method: "DELETE"
            }).then(res => {
                if (res.ok) {
                    console.log("Deleted " + shortUrl);
                } else {
                    console.log("Unable to delete " + shortUrl);
                }
                refreshData();
            });
        }
    };
    td.setAttribute("name", "deleteBtn");
    td.setAttribute("label", "Delete");
    div.appendChild(btn);
    td.appendChild(div);
    return td;
}

const editButton = (shortUrl) => {
    const td = document.createElement("td");
    const btn = document.createElement("button");
    btn.innerHTML = "Edit";

    btn.onclick = async (e) => {
        e.preventDefault();
        const newUrl = prompt("Enter the new long URL:");

        if (newUrl) {
            if (!isValidUrl(newUrl)) {
                showAlert("Invalid URL format. Please enter a valid URL.", "red");
                return;
            }

            const response = await fetch(prepSubdir(`/api/edit/${shortUrl}`), {
                method: "PUT",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ longlink: newUrl }),
            });

            if (response.ok) {
                showAlert(`Successfully updated ${shortUrl}.`, "green");
                refreshData();
            } else {
                const errorMsg = await response.text();
                showAlert(`Error: ${errorMsg}`, "red");
            }
        }
    };

    td.setAttribute("name", "editBtn");
    td.appendChild(btn);
    return td;
}

const isValidUrl = (urlString) => {
    try {
        new URL(urlString);
        return true;
    } catch (_) {
        return false;  
    }
}

const TD = (s, u) => {
    const td = document.createElement("td");
    const div = document.createElement("div");
    div.innerHTML = s;
    td.appendChild(div);
    td.setAttribute("label", u);
    return td;
}

const submitForm = () => {
    const form = document.forms.namedItem("new-url-form");
    const data = {
        "longlink": form.elements["longUrl"].value,
        "shortlink": form.elements["shortUrl"].value,
    };

    const url = prepSubdir("/api/new");
    let ok = false;

    fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(data),
    })
        .then(res => {
            ok = res.ok;
            return res.text();
        })
        .then(text => {
            if (!ok) {
                showAlert(text, "red");
            }
            else {
                copyShortUrl(text);
                longUrl.value = "";
                shortUrl.value = "";
                refreshData();
            }
        })
}

const submitLogin = () => {
    const password = document.getElementById("password");
    fetch(prepSubdir("/api/login"), {
        method: "POST",
        body: password.value
    }).then(res => {
        if (res.ok) {
            document.getElementById("container").style.filter = "blur(0px)"
            document.getElementById("login-dialog").close();
            password.value = '';
            refreshData();
        } else {
            const wrongPassBox = document.getElementById("wrong-pass");
            wrongPassBox.innerHTML = "Wrong password!";
            wrongPassBox.style.color = "red";
            password.focus();
        }
    })
}

const fetchApiKey = async () => {
    const response = await fetch(prepSubdir("/api/key"), {
        method: "POST"
    });

    if (response.ok) {
        const apiKey = await response.text();
        prompt("API Key:", apiKey);
    } else {
        const error_text = await response.text();
        alert("Failed to fetch API Key: " + error_text);
    }
}

const fetchResetApiKey = async () => {
    const response = await fetch(prepSubdir("/api/key"), {
        method: "DELETE"
    });

    if (response.ok) {
        const apiKey = await response.text();
        showAlert("Reset API Key Successfully with response: "+apiKey, "green");
    } else {
        const error_text = await response.text();
        alert("Failed to fetch Reset API Key: " + error_text);
    }
}

document.addEventListener("DOMContentLoaded", () => {
    const adminButton = document.getElementById("api-key-button");
    if (adminButton) {
        adminButton.addEventListener("click", (e) => {
            e.preventDefault();
            fetchApiKey();
        });
    }
});

document.addEventListener("DOMContentLoaded", () => {
    const adminButton = document.getElementById("reset-api-key-button");
    if (adminButton) {
        adminButton.addEventListener("click", (e) => {
            e.preventDefault();
            if (confirm("Do you want to reset API Key?")) {
                fetchResetApiKey();
            }
        });
    }
});

const logOut = async () => {
    let reply = await fetch(prepSubdir("/api/logout"), {method: "DELETE"}).then(res => res.text());
    console.log(reply);
    document.getElementById("table-box").hidden = true;
    document.getElementById("loading-text").hidden = false;
    refreshData();
}

(async () => {
    await refreshData();

    const form = document.forms.namedItem("new-url-form");
    form.onsubmit = e => {
        e.preventDefault();
        submitForm();
    }

    const login_form = document.forms.namedItem("login-form");
    login_form.onsubmit = e => {
        e.preventDefault();
        submitLogin();
    }
})()
