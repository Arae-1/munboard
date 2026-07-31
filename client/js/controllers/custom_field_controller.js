"use strict";

const router = require("../router.js");
const api = require("../api.js");
const misc = require("../util/misc.js");
const uri = require("../util/uri.js");
const CustomField = require("../models/custom_field.js");
const topNavigation = require("../models/top_navigation.js");
const CustomFieldView = require("../views/custom_field_view.js");
const EmptyView = require("../views/empty_view.js");

class CustomFieldController {
    constructor(ctx, section) {
        if (!api.hasPrivilege("custom_field_view")) {
            this._view = new EmptyView();
            this._view.showError("You don't have privileges to view custom fields.");
            return;
        }

        CustomField.get(ctx.parameters.id).then(
            (customField) => {
                topNavigation.activate("custom-fields");
                topNavigation.setTitle("Custom Field: " + customField.name);

                this._name = customField.name;
                customField.addEventListener("change", (e) =>
                    this._evtSaved(e, section)
                );

                this._view = new CustomFieldView({
                    customField: customField,
                    section: section,
                    customFieldDefaultColor: api.getCustomFieldDefaultColor(),
                    canEditAnything: api.hasPrivilege("custom_field_edit_name"),
                    canEditName: api.hasPrivilege("custom_field_edit_name"),
                    canEditDescription: api.hasPrivilege("custom_field_edit_name"),
                    canEditColor: api.hasPrivilege("custom_field_edit_name"),
                    canDelete: api.hasPrivilege("custom_field_delete"),
                });

                this._view.addEventListener("change", (e) =>
                    this._evtChange(e)
                );
                this._view.addEventListener("submit", (e) =>
                    this._evtUpdate(e)
                );
                this._view.addEventListener("delete", (e) =>
                    this._evtDelete(e)
                );
            },
            (error) => {
                this._view = new EmptyView();
                this._view.showError(error.message);
            }
        );
    }

    _evtChange(e) {
        misc.enableExitConfirmation();
    }

    _evtSaved(e, section) {
        misc.disableExitConfirmation();
        if (this._name !== e.detail.customField.name) {
            router.replace(
                uri.formatClientLink("custom-field", e.detail.customField.id, section),
                null,
                false
            );
        }
    }

    _evtUpdate(e) {
        this._view.clearMessages();
        this._view.disableForm();
        if (e.detail.name !== undefined && e.detail.name !== null) {
            e.detail.customField.name = e.detail.name;
        }
        if (e.detail.description !== undefined && e.detail.description !== null) {
            e.detail.customField.description = e.detail.description;
        }
        if (e.detail.color !== undefined && e.detail.color !== null) {
            e.detail.customField.color = e.detail.color;
        }
        e.detail.customField.save().then(
            () => {
                this._view.showSuccess("Custom field saved.");
                this._view.enableForm();
            },
            (error) => {
                this._view.showError(error.message);
                this._view.enableForm();
            }
        );
    }

    _evtDelete(e) {
        this._view.clearMessages();
        this._view.disableForm();
        e.detail.customField.delete().then(
            () => {
                const ctx = router.show(uri.formatClientLink("custom-fields"));
                ctx.controller.showSuccess("Custom field deleted.");
            },
            (error) => {
                this._view.showError(error.message);
                this._view.enableForm();
            }
        );
    }
}

module.exports = (router) => {
    router.enter(["custom-field", ":id", "edit"], (ctx, next) => {
        ctx.controller = new CustomFieldController(ctx, "edit");
    });
    router.enter(["custom-field", ":id", "delete"], (ctx, next) => {
        ctx.controller = new CustomFieldController(ctx, "delete");
    });
    router.enter(["custom-field", ":id"], (ctx, next) => {
        ctx.controller = new CustomFieldController(ctx, "summary");
    });
};
