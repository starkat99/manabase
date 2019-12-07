// Can't use JQuery, isn't loaded yet
function addCSSRule(rule) {
    var style = document.createElement("style");
    style.setAttribute("type", "text/css");
    style.innerText = rule;
    document.head.appendChild(style);
}

const INITIALLY_HIDDEN_FILTERS = ["silver-border"];
const INITIALLY_SHOWN_FILTERS = ["lands", "rocks", "dorks", "ramp"];

var show_filters = [];
var hide_filters = [];
var cookie_params = Cookies.getJSON("filter");
if (cookie_params !== undefined) {
    show_filters = show_filters.concat(cookie_params["show"]);
    hide_filters = hide_filters.concat(cookie_params["hide"]);
}

var params = new URLSearchParams(window.location.search);
show_filters = show_filters.concat(params.getAll("show"));
hide_filters = hide_filters.concat(params.getAll("hide"));

INITIALLY_HIDDEN_FILTERS.forEach(filter => {
    if (hide_filters.includes(filter)) {
        if (show_filters.includes(filter)) {
            show_filters = show_filters.filter(f => f != filter);
        }
        hide_filters = hide_filters.filter(f => f != filter);
    }
});

INITIALLY_SHOWN_FILTERS.forEach(filter => {
    if (show_filters.includes(filter)) {
        if (hide_filters.includes(filter)) {
            hide_filters = show_filters.filter(f => f != filter);
        }
        show_filters = hide_filters.filter(f => f != filter);
    }
});

function updateFilters() {
    document.querySelectorAll("head style").forEach(element => {
        element.remove();
    });
    INITIALLY_HIDDEN_FILTERS.concat(hide_filters).forEach(filter => {
        if (!show_filters.includes(filter)) {
            addCSSRule(".mtg-filter-" + filter + " { display: none !important; }");
        }
    });
}
updateFilters();