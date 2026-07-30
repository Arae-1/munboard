<div class='content-wrapper custom-field-create'>
    <form>
        <ul class='input'>
            <li class='name'>
                <%= ctx.makeTextInput({
                    text: 'Name',
                    value: '',
                    required: true,
                }) %>
            </li>
            <li class='description'>
                <%= ctx.makeTextarea({
                    text: 'Description',
                    value: '',
                }) %>
            </li>
            <li class='color'>
                <%= ctx.makeTextInput({
                    text: 'Color',
                    value: '',
                    placeholder: 'e.g. #ff0000 or red',
                }) %>
            </li>
        </ul>

        <% if (ctx.canCreate) { %>
            <div class='messages'></div>

            <div class='buttons'>
                <input type='submit' class='save' value='Create custom field'>
            </div>
        <% } %>
    </form>
</div>
