$("[data-component='card-transform-button']").each(function () {
    var target = $(this).data("target");
    $(target).flip({ trigger: 'manual' });
    $(this).on("click.manabase", function () {
        $(target).flip('toggle');
    });
});