<div class='content-wrapper' id='pool'>
    <nav class='pool-breadcrumbs' aria-label='Pool path'>
        <a href='<%- ctx.formatClientLink('pools') %>'>Pools</a>
        <% for (let ancestor of ctx.pool.ancestors) { %>
            <span aria-hidden='true'>/</span><a href='<%- ctx.formatClientLink('pool', ancestor.id) %>'><%- ctx.getPrettyName(ancestor.name) %></a>
        <% } %>
        <span aria-hidden='true'>/</span><span aria-current='page'><%- ctx.getPrettyName(ctx.pool.names[0]) %></span>
    </nav>
    <h1><%- ctx.getPrettyName(ctx.pool.names[0]) %></h1>
    <nav class='buttons'><!--
        --><ul><!--
            --><li data-name='summary'><a href='<%- ctx.formatClientLink('pool', ctx.pool.id) %>'>Summary</a></li><!--
            --><% if (ctx.canEditAnything) { %><!--
                --><li data-name='edit'><a href='<%- ctx.formatClientLink('pool', ctx.pool.id, 'edit') %>'>Edit</a></li><!--
            --><% } %><!--
            --><% if (ctx.canMerge) { %><!--
                --><li data-name='merge'><a href='<%- ctx.formatClientLink('pool', ctx.pool.id, 'merge') %>'>Merge with&hellip;</a></li><!--
            --><% } %><!--
            --><% if (ctx.canDelete) { %><!--
                --><li data-name='delete'><a href='<%- ctx.formatClientLink('pool', ctx.pool.id, 'delete') %>'>Delete</a></li><!--
            --><% } %><!--
        --></ul><!--
    --></nav>
    <div class='pool-content-holder'></div>
</div>
