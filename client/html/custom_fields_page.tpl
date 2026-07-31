<div class='custom-field-list table-wrap'>
    <% if (ctx.response.results.length) { %>
        <table>
            <thead>
                <th class='name'>
                    <% if (ctx.parameters.query == 'sort:name' || !ctx.parameters.query) { %>
                        <a href='<%- ctx.formatClientLink('custom-fields', {query: '-sort:name'}) %>'>Name</a>
                    <% } else { %>
                        <a href='<%- ctx.formatClientLink('custom-fields', {query: 'sort:name'}) %>'>Name</a>
                    <% } %>
                </th>
                <th class='description'>
                    <% if (ctx.parameters.query == 'sort:description') { %>
                        <a href='<%- ctx.formatClientLink('custom-fields', {query: '-sort:description'}) %>'>Description</a>
                    <% } else { %>
                        <a href='<%- ctx.formatClientLink('custom-fields', {query: 'sort:description'}) %>'>Description</a>
                    <% } %>
                </th>
                <th class='creation-time'>
                    <% if (ctx.parameters.query == 'sort:creation-time') { %>
                        <a href='<%- ctx.formatClientLink('custom-fields', {query: '-sort:creation-time'}) %>'>Created on</a>
                    <% } else { %>
                        <a href='<%- ctx.formatClientLink('custom-fields', {query: 'sort:creation-time'}) %>'>Created on</a>
                    <% } %>
                </th>
            </thead>
            <tbody>
                <% for (let cf of ctx.response.results) { %>
                    <tr>
                        <td class='name'>
                            <ul>
                                <li><a href='<%- ctx.formatClientLink('custom-field', cf.id) %>'<% if (cf.color) { %> style='color:<%- cf.color %>'<% } %>><%- cf.name %></a></li>
                            </ul>
                        </td>
                        <td class='description'>
                            <%- cf.description || '-' %>
                        </td>
                        <td class='creation-time'>
                            <%= ctx.makeRelativeTime(cf.creationTime) %>
                        </td>
                    </tr>
                <% } %>
            </tbody>
        </table>
    <% } %>
</div>
