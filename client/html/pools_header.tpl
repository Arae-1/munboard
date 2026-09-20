<div class='pool-list-header'>
    <form class='horizontal'>
        <ul class='input'>
            <li>
                <%= ctx.makeTextInput({text: 'Search query', id: 'search-text', name: 'search-text', value: ctx.parameters.query}) %>
            </li>
        </ul>

        <div class='buttons'>
            <input type='submit' value='Search'/>
            <a class='button append' href='<%- ctx.formatClientLink('help', 'search', 'pools') %>'>Syntax help</a>
            <% if (ctx.parameters.flat) { %>
                <a class='append' href='<%- ctx.formatClientLink('pools') %>'>Hierarchy view</a>
            <% } else { %>
                <a class='append' href='<%- ctx.formatClientLink('pools', {flat: 1}) %>'>Flat list</a>
            <% } %>

            <% if (ctx.canCreate) { %>
                <a class='append' href='<%- ctx.formatClientLink('pool', 'create') %>'>Add new pool</a>
            <% } %>

            <% if (ctx.canEditPoolCategories) { %>
                <a class='append' href='<%- ctx.formatClientLink('pool-categories') %>'>Pool categories</a>
            <% } %>
        </div>
    </form>
</div>
