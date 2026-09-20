<div class='content-wrapper pool-summary'>
    <nav class='buttons pool-post-navigation' aria-label='Post actions'>
        <ul>
            <li><a href='<%- ctx.formatClientLink('posts', {query: 'pool-tree:' + ctx.pool.searchableName}) %>'>Posts in this branch (<%- ctx.pool.postCount %>)</a></li>
            <% if (ctx.pool.directPosts.length !== ctx.pool.postCount) { %>
                <li><a href='<%- ctx.formatClientLink('posts', {query: 'pool:' + ctx.pool.searchableName}) %>'>Posts directly here (<%- ctx.pool.directPosts.length %>)</a></li>
            <% } %>
        </ul>
    </nav>

    <% if (ctx.pool.children.length) { %>
        <section class='children'>
            <hr/>
            <h2>Child pools</h2>
            <ul class='compact-pools child-pool-list'>
                <% for (let child of ctx.pool.children) { %>
                    <li>
                        <%= ctx.makePoolLink(child.id, false, false, child, child.name) %>
                        <a class='pool-usages' href='<%- ctx.formatClientLink('posts', {query: 'pool-tree:' + child.name}) %>' data-pseudo-content='<%- child.postCount %> post(s)'></a>
                    </li>
                <% } %>
            </ul>
        </section>
    <% } %>

    <% if (ctx.pool.description && ctx.pool.description.trim()) { %>
        <section class='description'>
            <hr/>
            <%= ctx.makeMarkdown(ctx.pool.description) %>
        </section>
    <% } %>

    <section class='details'>
        <h2>Details</h2>
        <section>
            Category:
            <span class='<%= ctx.makeCssName(ctx.pool.category, 'pool') %>'><%- ctx.pool.category %></span>
        </section>

        <% if (ctx.pool.names.slice(1).length) { %>
            <section>
            Aliases:<br/>
            <ul><!--
                --><% for (let name of ctx.pool.names.slice(1)) { %><!--
                    --><li><%= ctx.makePoolLink(ctx.pool.id, false, false, ctx.pool, name) %></li><!--
                --><% } %><!--
            --></ul>
            </section>
        <% } %>

        <% if (ctx.pool.launchDate) { %>
            <section>
                Launch Date:
                <span><%- ctx.pool.launchDate.slice(0, 10) %></span>
            </section>
        <% } %>
    </section>
</div>
