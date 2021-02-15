function bindEventsTo (root) {
    $(root).find('.toggle-btn').data('toggled', {on: true});

    $(root).find('.toggle-btn').click((e) => {
        $( $(e.target).attr('toggle-element') ).toggle();

        if ( $(e.target).data('toggled').on ) {
            $(e.target).data('toggled', {on: false});
            $(e.target).text(  $(e.target).attr('off-label') );

        }else {
            $(e.target).data('toggled', {on: true});
            $(e.target).text(  $(e.target).attr('on-label') );
        }
    });

    $(root).find('.http-btn').click((e) => {
        var method = $(e.target).attr('method');
        var url = $(e.target).attr('url');
        var data = eval( $(e.target).attr('data') );
        var onresponse = $(e.target).attr('onresponse');

        var obj = {
            method: method,
            url: url,
            data: data
        };

        var beforesend = $(e.target).attr('beforesend');

        if ( beforesend ) {
            obj = window[beforesend](obj);
        }

        $.ajax({
            method: obj.method,
            url: obj.url,
            data: JSON.stringify(obj.data),
            contentType: "application/json",
            complete: ()=> {
                eval( onresponse );
            }
        });

    });

    $(root).find('.remove-elem-btn').click((e) => {
        var elemToRemove = $(e.target).attr('elem');
        console.log(elemToRemove);
        if (elemToRemove === "this") {
            $(e.target).remove();

        }else if (elemToRemove === "parent") {

            $(e.target).parent().remove();

        }else {
            $(elemToRemove).remove();
        }
    });

    $(root).find('.make-elem-btn').click((e) => {
        var templateElementHtml = $( $(e.target).attr('template-selector') ).html();
        var containerElement = $( $(e.target).attr('container-selector') );
        var newElement = $(templateElementHtml);

        $(newElement).find('span.selector').each((i, selector) => {
            var attr = $(selector).attr('attr');
            var elem = $( $(selector).text() );
            var parent = $(selector).parent();

            console.log(parent, attr, elem);

            var data = null;
            if (attr === "value") {
                data = $(elem).val();

            }else if (attr == "text") {
                data = $(elem).text();

            }else if (attr == "html") {
                data = $(elem).html();

            }else {
                data = $(elem).attr(attr);
            }

            var targetElement = $(selector).attr('target');
            var targetElementAttr = $(selector).attr('target-attr');

            if (targetElement) {
                if (targetElementAttr) {
                    if (targetElementAttr === "value") {
                        $(targetElement).val(targetElementAttr, data);

                    }else if (targetElementAttr === "text") {
                        $(targetElement).text(targetElementAttr, data);

                    }else if (targetElementAttr === "html") {
                        $(targetElement).html(targetElementAttr, data);

                    }else {
                        $(targetElement).attr(targetElementAttr, data);
                    }
                }else {
                    $(targetElement).text(data)
                }

            }else {
                $(parent).text(data);
            }

            $(selector).remove();
        });

        bindEventsTo(newElement);
        $(containerElement).append(newElement);
    });
}

$(() => {
    bindEventsTo(document);
});

function serializeInputGroups(inputGroupsParentSelector) {
    var obj = {};

    $(inputGroupsParentSelector+' .input-group').each((i, inputGroup) => {
        var fieldName = null;

        if ( $(inputGroup).attr('field-name') ) {
            fieldName = $(inputGroup).attr('field-name');

        }else if ( $(inputGroup).attr('field-name-input')  ) {
            fieldName = $(inputGroup).find( $(inputGroup).attr('field-name-input') ).val();

        }else {
            fieldName = $(inputGroup).children('.input-group-text').text();
        }

        var fieldValue = null;
        if ( $(inputGroup).attr('field-value') ) {
            fieldValue = $(inputGroup).attr('field-value');

        }else if ( $(inputGroup).attr('field-value-input')  ) {
            fieldValue = $(inputGroup).find( $(inputGroup).attr('field-value-input') ).val();

        }else {
            fieldValue = $(inputGroup).find('input').val();
        }

        var dataType = $(inputGroup).children('input').attr('type');

        if (dataType === 'number') {
            var step = $(inputGroup).children('input').attr('step');

            if (step === '1') {
                obj[fieldName] = parseInt(fieldValue);
            }else {
                obj[fieldName] = parseFloat(fieldValue);
            }

        }else if (dataType === 'checkbox') {
            if ($(inputGroup).children('input').is(":checked")) {
                obj[fieldName] = true;
            }else {
                obj[fieldName] = false;
            }

        }else {
            obj[fieldName] = fieldValue;
        }
    });

    return obj;
}