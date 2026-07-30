"use strict";

const router = require("../router.js");
const api = require("../api.js");
const misc = require("../util/misc.js");
const uri = require("../util/uri.js");
const CustomFieldCreateView = require("../views/custom_field_create_view.js");
const EmptyView = require("../views/empty_view.js");

class CustomFieldCreateController {
    constructor(ctx) {
        if (!api.hasPrivilege("custom_field_create")) {
            this._view = new EmptyView();
            this._view.showError("You don't have privileges to create custom fields.");
            return;
        }

        this._view = new CustomFieldCreateView({
            canCreate: api.hasPrivilege("custom_field_create"),
        });

        this._view.addEventListener("submit", (e) =>
            this._evtCreate(e)
        );
    }

    _evtCreate(e) {
        this._view.clearMessages();
        this._view.disableForm();
        api.post(uri.formatApiLink("custom-fields"), e.detail).then(
            () => {
                this._view.clearMessages();
                misc.disableExitConfirmation();
                const ctx = router.show(uri.formatClientLink("custom-fields"));
                ctx.controller.showSuccess("Custom field created.");
            },
            (error) => {
                this._view.showError(error.message);
                this._view.enableForm();
            }
        );
    }
}

module.exports = (router) => {
    router.enter(["custom-field", "create"], (ctx, next) => {
        ctx.controller = new CustomFieldCreateController(ctx);
    });
};
