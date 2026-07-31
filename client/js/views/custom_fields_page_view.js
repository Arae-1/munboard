"use strict";

const api = require("../api.js");
const views = require("../util/views.js");

const template = views.getTemplate("custom-fields-page");

class CustomFieldsPageView {
    constructor(ctx) {
        ctx.customFieldDefaultColor = api.getCustomFieldDefaultColor();
        views.replaceContent(ctx.hostNode, template(ctx));
    }
}

module.exports = CustomFieldsPageView;
