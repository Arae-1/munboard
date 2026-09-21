<div class='content-wrapper pool-edit'>
    <form>
        <ul class='input'>
            <li class='names'>
                <% if (ctx.canEditNames) { %>
                    <%= ctx.makeTextInput({
                        text: 'Names',
                        value: ctx.pool.names.join(' '),
                        required: true,
                    }) %>
                <% } %>
            </li>
            <li class='category'>
                <% if (ctx.canEditCategory) { %>
                    <%= ctx.makeSelect({
                        text: 'Category',
                        keyValues: ctx.categories,
                        selectedKey: ctx.pool.category,
                        required: true,
                    }) %>
                <% } %>
            </li>
            <li class='description'>
                <% if (ctx.canEditDescription) { %>
                    <%= ctx.makeTextarea({
                        text: 'Description',
                        value: ctx.pool.description,
                    }) %>
                <% } %>
            </li>
            <% if (ctx.canEditLaunchDate) { %>
                <li class='launch-date'>
                    <%= ctx.makeDateInput({
                        text: 'Launch date (UTC)',
                        name: 'launch-date',
                        value: ctx.pool.launchDate ? ctx.pool.launchDate.slice(0, 10) : '',
                    }) %>
                </li>
            <% } %>
            <li class='posts'>
                <% if (ctx.canEditPosts) { %>
                    <%= ctx.makeTextInput({
                        text: 'Posts',
                        placeholder: 'space-separated post IDs',
                        value: ctx.pool.directPosts.map(post => post.id).join(' ')
                    }) %>
                <% } %>
            </li>
            <li class='children'>
                <% if (ctx.canEditPosts) { %>
                    <%= ctx.makeTextInput({
                        text: 'Child pools',
                        placeholder: 'type a pool name or #ID to add…'
                    }) %>
                    <p class='hint' id='child-pools-hint'>Select a suggested pool, or press Enter after a pool ID such as #7. Use the arrows to set the child order.</p>
                <% } %>
            </li>
        </ul>

        <% if (ctx.canEditAnything) { %>
            <div class='messages'></div>

            <div class='buttons'>
                <input type='submit' class='save' value='Save changes'>
            </div>
        <% } %>
    </form>
</div>
