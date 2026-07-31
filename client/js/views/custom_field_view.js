"use strict";

const events = require("../events.js");
const views = require("../util/views.js");
const CustomFieldSummaryView = require("./custom_field_summary_view.js");
const CustomFieldEditView = require("./custom_field_edit_view.js");
const CustomFieldDeleteView = require("./custom_field_delete_view.js");
const EmptyView = require("../views/empty_view.js");

const template = views.getTemplate("custom-field");

class CustomFieldView extends events.EventTarget {
    constructor(ctx) {
        super();

        this._ctx = ctx;
        ctx.customField.addEventListener("change", (e) => this._evtChange(e));
        ctx.section = ctx.section || "summary";

        this._hostNode = document.getElementById("content-holder");
        this._install();
    }

    _install() {
        const ctx = this._ctx;
        views.replaceContent(this._hostNode, template(ctx));

        for (let item of this._hostNode.querySelectorAll("[data-name]")) {
            item.classList.toggle(
                "active",
                item.getAttribute("data-name") === ctx.section
            );
            if (item.getAttribute("data-name") === ctx.section) {
                item.parentNode.scrollLeft =
                    item.getBoundingClientRect().left -
                    item.parentNode.getBoundingClientRect().left;
            }
        }

        ctx.hostNode = this._hostNode.querySelector(".custom-field-content-holder");
        if (ctx.section === "edit") {
            if (!this._ctx.canEditAnything) {
                this._view = new EmptyView();
                this._view.showError(
                    "You don't have privileges to edit custom fields."
                );
            } else {
                this._view = new CustomFieldEditView(ctx);
                events.proxyEvent(this._view, this, "submit");
            }
        } else if (ctx.section === "delete") {
            if (!this._ctx.canDelete) {
                this._view = new EmptyView();
                this._view.showError(
                    "You don't have privileges to delete custom fields."
                );
            } else {
                this._view = new CustomFieldDeleteView(ctx);
                events.proxyEvent(this._view, this, "submit", "delete");
            }
        } else {
            this._view = new CustomFieldSummaryView(ctx);
        }

        events.proxyEvent(this._view, this, "change");
        views.syncScrollPosition();
    }

    clearMessages() {
        this._view.clearMessages();
    }

    enableForm() {
        this._view.enableForm();
    }

    disableForm() {
        this._view.disableForm();
    }

    showSuccess(message) {
        this._view.showSuccess(message);
    }

    showError(message) {
        this._view.showError(message);
    }

    _evtChange(e) {
        this._ctx.customField = e.detail.customField;
        this._install(this._ctx);
    }
}

module.exports = CustomFieldView;
