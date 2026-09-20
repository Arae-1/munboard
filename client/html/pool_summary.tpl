<div class='content-wrapper pool-summary'>
    <section class='details'>
        <section>
            Category:
            <span class='<%= ctx.makeCssName(ctx.pool.category, 'pool') %>'><%- ctx.pool.category %></span>
        </section>

        <section>
        Aliases:<br/>
        <ul><!--
            --><% for (let name of ctx.pool.names.slice(1)) { %><!--
                --><li><%= ctx.makePoolLink(ctx.pool.id, false, false, ctx.pool, name) %></li><!--
            --><% } %><!--
        --></ul>
        </section>

    </section>

    <section class='description'>
        <hr/>
        <%= ctx.makeMarkdown(ctx.pool.description || 'This pool has no description yet.') %>
    </section>

    <nav class='buttons pool-post-navigation' aria-label='Post scope'>
        <ul>
            <li><a href='<%- ctx.formatClientLink('posts', {query: 'pool-tree:' + ctx.pool.searchableName}) %>'>All posts (<%- ctx.pool.postCount %>)</a></li>
            <li><a href='<%- ctx.formatClientLink('posts', {query: 'pool:' + ctx.pool.searchableName}) %>'>This pool only</a></li>
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

    <% if (ctx.pool.launchDate) { %>
        <section>
            <b>Launch Date:</b>
            <span><%- ctx.pool.launchDate.slice(0, 10) %></span>
        </section>
    <% } %>
</div>
