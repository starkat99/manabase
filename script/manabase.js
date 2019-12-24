$("[data-component='card-transform-button']").each(function () {
    var target = $(this).data("target");
    $(target).flip({ trigger: 'manual' });
    $(this).on("click.manabase", function () {
        $(target).flip('toggle');
    });
});

INITIALLY_SHOWN_FILTERS.concat(show_filters).forEach(filter => {
    if (!hide_filters.includes(filter)) {
        $("#filter-" + filter).prop("checked", "checked")
    }
});

$("input.mtg-filter").on("change", function () {
    var filter = $(this).attr("id").replace("filter-", "");
    var checked = $(this).is(":checked");
    if (checked) {
        if (INITIALLY_HIDDEN_FILTERS.includes(filter) && !show_filters.includes(filter)) {
            show_filters.push(filter);
        } else {
            hide_filters = hide_filters.filter(f => f != filter);
        }
    } else {
        if (INITIALLY_HIDDEN_FILTERS.includes(filter)) {
            show_filters = show_filters.filter(f => f != filter);
        } else if (!hide_filters.includes(filter)) {
            hide_filters.push(filter);
        }
    }
    updateFilters();
});