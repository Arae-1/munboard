<div class='content-wrapper custom-field-edit'>
    <form>
        <ul class='input'>
            <% if (ctx.canEditName) { %>
                <li class='name'>
                    <%= ctx.makeTextInput({
                        text: 'Name',
                        value: ctx.customField.name,
                        required: true,
                    }) %>
                </li>
            <% } %>
            <% if (ctx.canEditDescription) { %>
                <li class='description'>
                    <%= ctx.makeTextarea({
                        text: 'Description',
                        value: ctx.customField.description,
                    }) %>
                </li>
            <% } %>
            <% if (ctx.canEditColor) { %>
                <li class='color'>
                    <%= ctx.makeTextInput({
                        text: 'Color',
                        value: ctx.customField.color,
                        placeholder: 'e.g. #ff0000 or red',
                    }) %>
                </li>
            <% } %>
        </ul>

        <% if (ctx.canEditAnything) { %>
            <div class='messages'></div>

            <div class='buttons'>
                <input type='submit' class='save' value='Save changes'>
            </div>
        <% } %>
    </form>
</div>
