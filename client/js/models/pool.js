"use strict";

const api = require("../api.js");
const uri = require("../util/uri.js");
const events = require("../events.js");
const misc = require("../util/misc.js");

class Pool extends events.EventTarget {
    constructor() {
        const PostList = require("./post_list.js");

        super();
        this._orig = {};

        for (let obj of [this, this._orig]) {
            obj._posts = new PostList();
            obj._directPosts = new PostList();
            obj._children = [];
        }

        this._updateFromResponse({});
    }

    get id() {
        return this._id;
    }

    get names() {
        return this._names;
    }

    get searchableName() {
        if (this._names.length > 0 && !/^\d+$/.test(this._names[0])) {
            return this._names[0];
        } else {
            return this._id;
        }
    }

    get category() {
        return this._category;
    }

    get description() {
        return this._description;
    }

    get posts() {
        return this._posts;
    }

    get postCount() {
        return this._postCount;
    }

    get directPosts() {
        return this._directPosts;
    }

    get parent() {
        return this._parent;
    }

    set parent(value) {
        this._parent = value;
    }

    get ancestors() {
        return this._ancestors || [];
    }

    get children() {
        return this._children;
    }

    set children(value) {
        this._children = value.map((id) =>
            this._children.find((child) => child.id === id) || { id: id }
        );
    }

    get creationTime() {
        return this._creationTime;
    }

    get lastEditTime() {
        return this._lastEditTime;
    }

    get launchDate() {
        return this._launchDate;
    }

    set names(value) {
        this._names = value;
    }

    set category(value) {
        this._category = value;
    }

    set description(value) {
        this._description = value;
    }

    set launchDate(value) {
        this._launchDate = value;
    }

    static fromResponse(response) {
        const ret = new Pool();
        ret._updateFromResponse(response);
        return ret;
    }

    static get(id, fields) {
        const args = ["pool", id];
        if (fields) {
            args.push({ fields: fields.join(",") });
        }
        return api.get(uri.formatApiLink(...args)).then((response) => {
            return Promise.resolve(Pool.fromResponse(response));
        });
    }

    save() {
        const detail = { version: this._version };

        // send only changed fields to avoid user privilege violation
        if (misc.arraysDiffer(this._names, this._orig._names, true)) {
            detail.names = this._names;
        }
        if (this._category !== this._orig._category) {
            detail.category = this._category;
        }
        if (this._description !== this._orig._description) {
            detail.description = this._description;
        }
        if (misc.arraysDiffer(this._directPosts, this._orig._directPosts)) {
            detail.posts = this._directPosts.map((post) => post.id);
        }
        if ((this._parent && this._parent.id) !== (this._orig._parent && this._orig._parent.id)) {
            detail.parent = this._parent ? this._parent.id : null;
        }
        if (misc.arraysDiffer(this._children.map((child) => child.id), this._orig._children.map((child) => child.id))) {
            detail.children = this._children.map((child) => child.id);
        }
        if (this._launchDate !== this._orig._launchDate) {
            detail.launchDate = this._launchDate;
        }

        let promise = this._id
            ? api.put(uri.formatApiLink("pool", this._id), detail)
            : api.post(uri.formatApiLink("pools"), detail);
        return promise.then((response) => {
            this._updateFromResponse(response);
            this.dispatchEvent(
                new CustomEvent("change", {
                    detail: {
                        pool: this,
                    },
                })
            );
            return Promise.resolve();
        });
    }

    merge(targetId, addAlias) {
        return api
            .get(uri.formatApiLink("pool", targetId))
            .then((response) => {
                return api.post(uri.formatApiLink("pool-merge"), {
                    removeVersion: this._version,
                    remove: this._id,
                    mergeToVersion: response.version,
                    mergeTo: targetId,
                });
            })
            .then((response) => {
                this._updateFromResponse(response);
                this.dispatchEvent(
                    new CustomEvent("change", {
                        detail: {
                            pool: this,
                        },
                    })
                );
                return Promise.resolve();
            });
    }

    delete() {
        return api
            .delete(uri.formatApiLink("pool", this._id), {
                version: this._version,
            })
            .then((response) => {
                this.dispatchEvent(
                    new CustomEvent("delete", {
                        detail: {
                            pool: this,
                        },
                    })
                );
                return Promise.resolve();
            });
    }

    _updateFromResponse(response) {
        const map = {
            _id: response.id,
            _version: response.version,
            _origName: response.names ? response.names[0] : null,
            _names: response.names,
            _category: response.category,
            _description: response.description,
            _creationTime: response.creationTime,
            _lastEditTime: response.lastEditTime,
            _launchDate: response.launchDate,
            _postCount: response.postCount || 0,
            _parent: response.parent || null,
            _ancestors: response.ancestors || [],
        };

        for (let obj of [this, this._orig]) {
            obj._posts.sync(response.posts);
            obj._directPosts.sync(response.directPosts || response.posts);
            obj._children = [...(response.children || [])];
        }

        Object.assign(this, map);
        Object.assign(this._orig, map);
    }
}

module.exports = Pool;
