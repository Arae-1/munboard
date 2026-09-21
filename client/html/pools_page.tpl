<div class='pool-list table-wrap <%= ctx.hierarchy ? 'hierarchy' : '' %>'>
    <% if (ctx.response.results.length) { %>
        <% if (ctx.hierarchy) { %>
            <% let children = {}; let rows = []; let added = {}; for (let pool of ctx.response.results) { let parentId = pool.parent ? pool.parent.id : null; (children[parentId] || (children[parentId] = [])).push(pool); } function addRows(parentId, depth, ancestorContinues) { let siblings = children[parentId] || []; for (let index = 0; index < siblings.length; index++) { let pool = siblings[index]; if (added[pool.id]) { continue; } let isLastChild = index == siblings.length - 1; added[pool.id] = true; rows.push({pool: pool, depth: depth, isLastChild: isLastChild, ancestorContinues: ancestorContinues}); addRows(pool.id, depth + 1, depth ? ancestorContinues.concat(!isLastChild) : []); } } addRows(null, 0, []); for (let pool of ctx.response.results) { if (!added[pool.id]) { added[pool.id] = true; rows.push({pool: pool, depth: 0, isLastChild: true, ancestorContinues: []}); addRows(pool.id, 1, []); } } %>
            <table>
                <colgroup>
                    <col class='names'/>
                    <col class='post-count'/>
                    <col class='creation-time'/>
                </colgroup>
                <thead>
                    <tr>
                        <th class='names'>
                            <% if (ctx.parameters.query == 'sort:name' || !ctx.parameters.query) { %>
                                <a href='<%- ctx.formatClientLink('pools', {query: '-sort:name'}) %>'>Pool name(s)</a>
                            <% } else { %>
                                <a href='<%- ctx.formatClientLink('pools', {query: 'sort:name'}) %>'>Pool name(s)</a>
                            <% } %>
                        </th>
                        <th class='post-count'>
                             <% if (ctx.parameters.query == 'sort:post-count') { %>
                                <a href='<%- ctx.formatClientLink('pools', {query: '-sort:post-count'}) %>'>Post count</a>
                             <% } else { %>
                                <a href='<%- ctx.formatClientLink('pools', {query: 'sort:post-count'}) %>'>Post count</a>
                             <% } %>
                        </th>
                        <th class='creation-time'>
                            <% if (ctx.parameters.query == 'sort:creation-time') { %>
                                <a href='<%- ctx.formatClientLink('pools', {query: '-sort:creation-time'}) %>'>Created on</a>
                            <% } else { %>
                                <a href='<%- ctx.formatClientLink('pools', {query: 'sort:creation-time'}) %>'>Created on</a>
                            <% } %>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    <% for (let row of rows) { %>
                        <tr class='<%- row.depth ? 'child' : 'root' %> depth-<%- row.depth %>'>
                            <td class='names'>
                                <div class='pool-row'>
                                    <% if (row.depth) { %>
                                        <span class='tree-guides' aria-hidden='true'><% for (let continues of row.ancestorContinues) { %><span class='tree-guide ancestor <%- continues ? 'continues' : '' %>'></span><% } %><span class='tree-guide connector <%- row.isLastChild ? 'last-child' : 'branch' %>'></span></span>
                                    <% } %>
                                    <ul class='pool-names'>
                                        <% for (let name of row.pool.names) { %>
                                            <li><a href='<%- ctx.formatClientLink('posts', {query: 'pool-tree:' + row.pool.searchableName}) %>' class='<%- ctx.makeCssName(row.pool.category, 'pool') %>'><%- ctx.getPrettyName(name) %></a></li>
                                        <% } %>
                                    </ul>
                                    <span class='pool-actions'><a class='pool-overview' href='<%- ctx.formatClientLink('pool', row.pool.id) %>'>Overview</a></span>
                                </div>
                            </td>
                            <td class='post-count'><a href='<%- ctx.formatClientLink('posts', {query: 'pool:' + row.pool.searchableName}) %>'><%- row.pool.postCount %></a></td>
                            <td class='creation-time'><%= ctx.makeRelativeTime(row.pool.creationTime) %></td>
                        </tr>
                    <% } %>
                </tbody>
            </table>
        <% } else { %>
        <table>
            <colgroup>
                <col class='names'/>
                <col class='post-count'/>
                <col class='creation-time'/>
            </colgroup>
            <thead>
                <tr>
                    <th class='names'>
                        <% if (ctx.parameters.query == 'sort:name' || !ctx.parameters.query) { %>
                            <a href='<%- ctx.formatClientLink('pools', {query: '-sort:name'}) %>'>Pool name(s)</a>
                        <% } else { %>
                            <a href='<%- ctx.formatClientLink('pools', {query: 'sort:name'}) %>'>Pool name(s)</a>
                        <% } %>
                    </th>
                    <th class='post-count'>
                         <% if (ctx.parameters.query == 'sort:post-count') { %>
                            <a href='<%- ctx.formatClientLink('pools', {query: '-sort:post-count'}) %>'>Post count</a>
                         <% } else { %>
                            <a href='<%- ctx.formatClientLink('pools', {query: 'sort:post-count'}) %>'>Post count</a>
                         <% } %>
                    </th>
                    <th class='creation-time'>
                        <% if (ctx.parameters.query == 'sort:creation-time') { %>
                            <a href='<%- ctx.formatClientLink('pools', {query: '-sort:creation-time'}) %>'>Created on</a>
                        <% } else { %>
                            <a href='<%- ctx.formatClientLink('pools', {query: 'sort:creation-time'}) %>'>Created on</a>
                        <% } %>
                    </th>
                </tr>
            </thead>
            <tbody>
                <% for (let pool of ctx.response.results) { %>
                    <tr>
                        <td class='names'>
                            <ul>
                                <% for (let name of pool.names) { %>
                                    <li><%= ctx.makePoolLink(pool.id, false, false, pool, name) %></li>
                                <% } %>
                            </ul>
                        </td>
                        <td class='post-count'>
                            <a href='<%- ctx.formatClientLink('posts', {query: 'pool:' + pool.searchableName}) %>'><%- pool.postCount %></a>
                        </td>
                        <td class='creation-time'>
                            <%= ctx.makeRelativeTime(pool.creationTime) %>
                        </td>
                    </tr>
                <% } %>
            </tbody>
        </table>
        <% } %>
    <% } %>
</div>
