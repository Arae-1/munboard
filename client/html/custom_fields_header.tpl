<div class='custom-field-list-header'>
    <form class='horizontal'>
        <ul class='input'>
            <li>
                <%= ctx.makeTextInput({text: 'Search query', id: 'search-text', name: 'search-text', value: ctx.parameters ? ctx.parameters.query : ''}) %>
            </li>
        </ul>

        <div class='buttons'>
            <input type='submit' value='Search'/>
            <a class='button append' href='<%- ctx.formatClientLink('help', 'search', 'custom-fields') %>'>Syntax help</a>
            <% if (ctx.canCreate) { %>
                <a class='append' href='<%- ctx.formatClientLink('custom-field', 'create') %>'>Add new custom field</a>
            <% } %>
        </div>
    </form>
</div>
