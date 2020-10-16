import Vue from "vue";
import VueRouter from "vue-router";
import Welcome from "../views/Welcome.vue";
import Login from "../views/Login.vue";
import Register from "../views/Register.vue";
import Dashboard from "../views/Dashboard.vue";
import ProjectView from "../components/ProjectView.vue";
import AddProject from "../components/AddProject.vue";

Vue.use(VueRouter);

const routes = [
  {
    path: "/",
    name: "Welcome",
    component: Welcome,
  },
  {
    path: "/login",
    name: "Login",
    component: Login,
  },
  {
    path: "/register",
    name: "Register",
    component: Register,
  },
  {
    path: "/dashboard",
    name: "Dashboard",
    component: Dashboard,
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
];

const router = new VueRouter({
  mode: "history",
  base: process.env.BASE_URL,
  routes,
});

router.beforeEach((to, from, next) => {
  let access_token = Vue.$cookies.get("token");
  // if (access_token === null) {
  //   if (to.name === "Login" || to.name === "Register" || to.name === "Welcome")
  //     next();
  //   else next({ name: "Login" });
  // } else {
  //   next();
  // }
  next();
});

export default router;
