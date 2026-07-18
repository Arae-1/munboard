"use strict";

const router = require("../router.js");
const api = require("../api.js");
const misc = require("../util/misc.js");
const uri = require("../util/uri.js");
const TagCategoryList = require("../models/tag_category_list.js");
const TagCreateView = require("../views/tag_create_view.js");
const EmptyView = require("../views/empty_view.js");

class TagCreateController {
    constructor(ctx) {
        if (!api.hasPrivilege("tag_create")) {
            this._view = new EmptyView();
            this._view.showError("You don't have privileges to create tags.");
            return;
        }

        TagCategoryList.get().then(
            (tagCategoriesResponse) => {
                const categories = {};
                for (let category of tagCategoriesResponse.results) {
                    categories[category.name] = category.name;
                }

                this._view = new TagCreateView({
                    canCreate: api.hasPrivilege("tag_create"),
                    categories: categories,
                    escapeTagName: uri.escapeTagName,
                });

                this._view.addEventListener("submit", (e) =>
                    this._evtCreate(e)
                );
            },
            (error) => {
                this._view = new EmptyView();
                this._view.showError(error.message);
            }
        );
    }

    _evtCreate(e) {
        this._view.clearMessages();
        this._view.disableForm();
        api.post(uri.formatApiLink("tags"), e.detail).then(
            () => {
                this._view.clearMessages();
                misc.disableExitConfirmation();
                const ctx = router.show(uri.formatClientLink("tags"));
                ctx.controller.showSuccess("Tag created.");
            },
            (error) => {
                this._view.showError(error.message);
                this._view.enableForm();
            }
        );
    }
}

module.exports = (router) => {
    router.enter(["tag", "create"], (ctx, next) => {
        ctx.controller = new TagCreateController(ctx, "create");
    });
};
