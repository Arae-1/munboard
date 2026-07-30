"use strict";

const views = require("../util/views.js");

const template = views.getTemplate("custom-fields-header");

class CustomFieldsHeaderView {
    constructor(ctx) {
        this._hostNode = ctx.hostNode;
        views.replaceContent(this._hostNode, template(ctx));
    }
}

module.exports = CustomFieldsHeaderView;
