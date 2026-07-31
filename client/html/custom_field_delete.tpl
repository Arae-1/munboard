<div class='custom-field-delete'>
    <form>
        <p>Are you sure you want to delete the custom field <strong style='color:<%- ctx.customField.color || customFieldDefaultColor %>'><%- ctx.customField.name %></strong>?</p>
        <p>This will also remove the field from all posts it's assigned to.</p>

        <ul class='input'>
            <li>
                <%= ctx.makeCheckbox({
                    name: 'confirm-deletion',
                    text: 'I confirm that I want to delete this custom field.',
                    required: true,
                }) %>
            </li>
        </ul>

        <div class='messages'></div>

        <div class='buttons'>
            <input type='submit' value='Delete custom field'/>
        </div>
    </form>
</div>
