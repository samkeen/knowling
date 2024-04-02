import {createRouter, createWebHistory} from "vue-router";
import HomeView from "../views/NotesList.vue";

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: [
        {
            path: "/",
            name: "Home",
            component: HomeView,
        },
        {
            path: "/note/:new", // :new is a fake/unused param, see AppHeader.vue
            name: "NewNote",
            // route level code-splitting (this gives us lazy loading so not all routes are loaded at once)
            component: () => import("../views/Editor.vue"),
        },
        {
            path: "/note/:id",
            name: "EditNote",
            component: () => import("../views/Editor.vue")
        },
        {
            path: "/settings",
            name: "Settings",
            component: () => import("../views/Settings.vue")
        },
    ],
});
export default router;
