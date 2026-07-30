"use strict";

const router = require("../router.js");
const api = require("../api.js");
const uri = require("../util/uri.js");
const CustomFieldList = require("../models/custom_field_list.js");
const topNavigation = require("../models/top_navigation.js");
const CustomFieldsHeaderView = require("../views/custom_fields_header_view.js");
const CustomFieldsPageView = require("../views/custom_fields_page_view.js");
const EmptyView = require("../views/empty_view.js");

const fields = [
    "id",
    "name",
    "description",
    "color",
    "creationTime",
];

class CustomFieldListController {
    constructor(ctx) {
        if (!api.hasPrivilege("custom_field_list")) {
            this._view = new EmptyView();
            this._view.showError("You don't have privileges to view custom fields.");
            return;
        }

        topNavigation.activate("custom-fields");
        topNavigation.setTitle("Listing custom fields");

        const hostNode = document.getElementById("content-holder");
        const pageHeaderHolderNode = document.createElement("div");
        pageHeaderHolderNode.classList.add("page-header-holder");
        const pageContentHolderNode = document.createElement("div");
        pageContentHolderNode.classList.add("page-content-holder");
        hostNode.innerHTML = "";
        hostNode.appendChild(pageHeaderHolderNode);
        hostNode.appendChild(pageContentHolderNode);

        this._headerView = new CustomFieldsHeaderView({
            hostNode: pageHeaderHolderNode,
            canCreate: api.hasPrivilege("custom_field_create"),
        });

        this._loadPage(pageContentHolderNode);
    }

    _loadPage(hostNode) {
        CustomFieldList.get(fields).then(
            (response) => {
                this._view = new CustomFieldsPageView({
                    hostNode: hostNode,
                    customFields: response.results,
                });
            },
            (error) => {
                this._view = new EmptyView();
                this._view.showError(error.message);
            }
        );
    }

    showSuccess(message) {
        const hostNode = document.getElementById("content-holder");
        const messagesNode = hostNode.querySelector(".messages");
        if (messagesNode) {
            const views = require("../util/views.js");
            views.showSuccess(messagesNode.parentNode, message);
        }
    }

    showError(message) {
        const hostNode = document.getElementById("content-holder");
        const messagesNode = hostNode.querySelector(".messages");
        if (messagesNode) {
            const views = require("../util/views.js");
            views.showError(messagesNode.parentNode, message);
        }
    }
}

module.exports = (router) => {
    router.enter(["custom-fields"], (ctx, next) => {
        ctx.controller = new CustomFieldListController(ctx);
    });
};
