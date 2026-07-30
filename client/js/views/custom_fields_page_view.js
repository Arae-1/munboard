"use strict";

const views = require("../util/views.js");

const template = views.getTemplate("custom-fields-page");

class CustomFieldsPageView {
    constructor(ctx) {
        views.replaceContent(ctx.hostNode, template(ctx));
    }
}

module.exports = CustomFieldsPageView;
