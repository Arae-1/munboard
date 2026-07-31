<div class='content-wrapper' id='custom-field'>
    <h1 style='color:<%- ctx.customField.color || ctx.customFieldDefaultColor %>'><%- ctx.customField.name %></h1>
    <nav class='buttons'><!--
        --><ul><!--
            --><li data-name='summary'><a href='<%- ctx.formatClientLink('custom-field', ctx.customField.id) %>'>Summary</a></li><!--
            --><% if (ctx.canEditAnything) { %><!--
                --><li data-name='edit'><a href='<%- ctx.formatClientLink('custom-field', ctx.customField.id, 'edit') %>'>Edit</a></li><!--
            --><% } %><!--
            --><% if (ctx.canDelete) { %><!--
                --><li data-name='delete'><a href='<%- ctx.formatClientLink('custom-field', ctx.customField.id, 'delete') %>'>Delete</a></li><!--
            --><% } %><!--
        --></ul><!--
    --></nav>
    <div class='custom-field-content-holder'></div>
</div>
