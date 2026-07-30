<div class='custom-field-list table-wrap'>
    <div class='messages'></div>
    <% if (ctx.customFields.length) { %>
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
                <th class='color'>
                    <% if (ctx.parameters.query == 'sort:color') { %>
                        <a href='<%- ctx.formatClientLink('custom-fields', {query: '-sort:color'}) %>'>Color</a>
                    <% } else { %>
                        <a href='<%- ctx.formatClientLink('custom-fields', {query: 'sort:color'}) %>'>Color</a>
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
                <% for (let cf of ctx.customFields) { %>
                    <tr>
                        <td class='name'>
                            <ul>
                                <li><a href='<%- ctx.formatClientLink('custom-field', cf.id) %>'><%- cf.name %></a></li>
                            </ul>
                        </td>
                        <td class='description'>
                            <%- cf.description ? cf.description.slice(0, 100) : '-' %>
                        </td>
                        <td class='color'>
                            <% if (cf.color) { %>
                                <span style='display:inline-block;width:12px;height:12px;background:<%- cf.color %>;vertical-align:middle;border:1px solid var(--border-color)'></span>
                                <%- cf.color %>
                            <% } else { %>
                                -
                            <% } %>
                        </td>
                        <td class='creation-time'>
                            <%= ctx.makeRelativeTime(cf.creationTime) %>
                        </td>
                    </tr>
                <% } %>
            </tbody>
        </table>
    <% } else { %>
        <p>No custom fields found.</p>
    <% } %>
</div>
