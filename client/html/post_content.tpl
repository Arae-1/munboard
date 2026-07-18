<div class='post-content post-type-<%- ctx.post.type %>' style='<%- ctx.post.type === 'flash' ? 'background-image: url('+ctx.post.thumbnailUrl+')' : '' %>'>
    <% if (['image', 'animation'].includes(ctx.post.type)) { %>

        <img class='resize-listener' alt='' src='<%- ctx.post.contentUrl %>' draggable='false'/>

    <% } else if (ctx.post.type === 'flash') { %>

        <object class='resize-listener' width='<%- ctx.post.canvasWidth %>' height='<%- ctx.post.canvasHeight %>' data='<%- ctx.post.contentUrl %>'>
            <param name='wmode' value='transparent'/>
            <param name='movie' value='<%- ctx.post.contentUrl %>'/>
            <div class='messages'><div class='message-wrapper'><div class='message error'>Your browser does not support Flash.</div></div></div>
        </object>

    <% } else if (ctx.post.type === 'video') { %>

        <%= ctx.makeElement(
            'video', {
                class: 'resize-listener',
                controls: true,
                loop: (ctx.post.flags || []).includes('loop'),
                playsinline: true,
                autoplay: ctx.autoplay,
            },
            ctx.makeElement('source', {
                type: ctx.post.mimeType,
                src: ctx.post.contentUrl,
            }),
            'Your browser doesn\'t support HTML5 videos.')
        %>

        <% } else if (ctx.post.type === 'document') { %>

            <div class='post-document-viewer'>
                <embed class='resize-listener' src='<%- ctx.post.contentUrl %>' type='application/pdf' width='100%' height='800px'/>
                <p><a href='<%- ctx.post.contentUrl %>' target='_blank'>Open in new tab</a></p>
            </div>

    <% } else { console.log(new Error('Unknown post type')); } %>

    <div class='post-overlay resize-listener'>
    </div>
</div>
