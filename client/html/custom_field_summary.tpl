<div class='content-wrapper custom-field-summary'>
    <section class='details'>
        <% if (ctx.customField.description) { %>
            <section>
                Description:<br/>
                <%- ctx.makeMarkdown(ctx.customField.description) %>
            </section>
        <% } %>

        <% if (ctx.customField.color) { %>
            <section>
                Color:
                <span style='display:inline-block;width:16px;height:16px;background:<%- ctx.customField.color %>;vertical-align:middle;border:1px solid var(--border-color)'></span>
                <%- ctx.customField.color %>
            </section>
        <% } %>
    </section>

    <section>
        <b>Created:</b> <%= ctx.makeRelativeTime(ctx.customField.creationTime) %>
    </section>
</div>
