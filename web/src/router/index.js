import Vue from "vue";
import VueRouter from "vue-router";
import Welcome from "../views/Welcome.vue";
import Login from "../views/Login.vue";
import Register from "../views/Register.vue";
import Pricing from "../views/Pricing.vue";
import Models from "../views/Models.vue";
import Confirm from "../views/Confirm.vue";
import Settings from "../views/Settings.vue";
import Dashboard from "../views/Dashboard.vue";
import KeyConfirmation from "../views/KeyConfirmation.vue";
import ProjectView from "../components/ProjectView.vue";
import AddProject from "../components/AddProject.vue";
import store from "../store";
import _ from "lodash";

Vue.use(VueRouter);

const routes = [
  {
    path: "/",
    name: "Welcome",
    component: Welcome,
    meta: { guest: true },
  },
  {
    path: "/login",
    name: "Login",
    component: Login,
    meta: { guest: true },
  },
  {
    path: "/register",
    name: "Register",
    component: Register,
    meta: { guest: true },
  },
  {
    path: "/pricing",
    name: "Pricing",
    component: Pricing,
  },
  {
    path: "/client/confirm",
    name: "Confirm",
    component: Confirm,
    meta: { requiresAuth: true },
  },
  {
    path: "/client/confirm/success",
    name: "PrivateKey",
    component: KeyConfirmation,
    meta: { requiresAuth: true },
    props: true,
  },
  {
    path: "/dashboard",
    name: "Dashboard",
    component: Dashboard,
    meta: { requiresAuth: true },
    children: [
      {
        path: "/dashboard/:projectId",
        name: "ProjectView",
        component: ProjectView,
        props: true,
      },
      {
        path: "/dashboard/new",
        name: "AddProject",
        component: AddProject,
      },
    ],
  },
  {
    path: "/settings",
    name: "Settings",
    component: Settings,
    meta: { requiresAuth: true },
  },
  {
    path: "/models",
    name: "Models",
    component: Models,
    meta: { requiresAuth: true },
  },
];

const router = new VueRouter({
  mode: "history",
  base: process.env.BASE_URL,
  routes,
});

router.beforeEach(async (to, from, next) => {
  let token = Vue.prototype.$cookies.get("token");
  if (to.matched.some((record) => record.meta.guest)) {
    if (store.getters.isAuthenticated) {
      next({ name: "Dashboard" });
      return;
    } else if (token) {
      let user_data = await store.dispatch("getUserData");

      let commit_payload = {
        name: user_data.data.first_name + " " + user_data.data.last_name,
        email: user_data.data.email,
        client: user_data.data.client,
        credits: user_data.data.credits,
      };

      store.commit("setUser", commit_payload);
      next({ name: "Dashboard" });
      return;
    }
    next();
  }
  next();
});

router.beforeEach(async (to, from, next) => {
  if (to.matched.some((record) => record.meta.requiresAuth)) {
    let token = Vue.prototype.$cookies.get("token");
    if (token) {
      if (!store.getters.isAuthenticated) {
        let user_data = await store.dispatch("getUserData");

        let commit_payload = {
          name: user_data.data.first_name + " " + user_data.data.last_name,
          email: user_data.data.email,
          client: user_data.data.client,
          credits: user_data.data.credits,
        };

        store.commit("setUser", commit_payload);
        store.dispatch("getAvatar");
      }
      next();
      return;
    } else {
      next("/login");
    }
  }
  next();
});

export default router;
