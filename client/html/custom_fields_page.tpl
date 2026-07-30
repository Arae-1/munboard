<div class='custom-field-list table-wrap'>
    <% if (ctx.customFields.length) { %>
        <table>
            <thead>
                <th class='name'>Name</th>
                <th class='description'>Description</th>
                <th class='color'>Color</th>
                <th class='creation-time'>Created on</th>
            </thead>
            <tbody>
                <% for (let cf of ctx.customFields) { %>
                    <tr>
                        <td class='name'>
                            <a href='<%- ctx.formatClientLink('custom-field', cf.id) %>'><%- cf.name %></a>
                        </td>
                        <td class='description'>
                            <%- cf.description ? cf.description.slice(0, 100) : '' %>
                        </td>
                        <td class='color'>
                            <% if (cf.color) { %>
                                <span style='display:inline-block;width:12px;height:12px;background:<%- cf.color %>;vertical-align:middle;border:1px solid var(--border-color)'></span>
                                <%- cf.color %>
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
        <p>No custom fields yet.</p>
    <% } %>
</div>
