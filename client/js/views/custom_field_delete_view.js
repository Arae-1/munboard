"use strict";

const events = require("../events.js");
const views = require("../util/views.js");

const template = views.getTemplate("custom-field-delete");

class CustomFieldDeleteView extends events.EventTarget {
    constructor(ctx) {
        super();

        this._hostNode = ctx.hostNode;
        this._customField = ctx.customField;
        views.replaceContent(this._hostNode, template(ctx));
        views.decorateValidator(this._formNode);
        this._formNode.addEventListener("submit", (e) => this._evtSubmit(e));
    }

    clearMessages() {
        views.clearMessages(this._hostNode);
    }

    enableForm() {
        views.enableForm(this._formNode);
    }

    disableForm() {
        views.disableForm(this._formNode);
    }

    showSuccess(message) {
        views.showSuccess(this._hostNode, message);
    }

    showError(message) {
        views.showError(this._hostNode, message);
    }

    _evtSubmit(e) {
        e.preventDefault();
        this.dispatchEvent(
            new CustomEvent("submit", {
                detail: {
                    customField: this._customField,
                },
            })
        );
    }

    get _formNode() {
        return this._hostNode.querySelector("form");
    }
}

module.exports = CustomFieldDeleteView;
