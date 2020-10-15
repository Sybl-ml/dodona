import Vue from "vue";
import App from "./App.vue";
import router from "./router";
import store from "./store";
import { BootstrapVue } from "bootstrap-vue";
import VueCookies from "vue-cookies";

// Install BootstrapVue
Vue.use(BootstrapVue);
Vue.use(VueCookies);

import "@/assets/css/custom.scss";

Vue.config.productionTip = false;

new Vue({
  router,
  store,
  render: (h) => h(App),
}).$mount("#app");
