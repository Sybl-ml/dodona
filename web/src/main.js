import Vue from "vue";
import App from "./App.vue";
import router from "./router";
import store from "./store";
import axios from "axios";
import { BootstrapVue } from "bootstrap-vue";

// Install BootstrapVue
Vue.use(BootstrapVue);

import "@/assets/css/custom.scss";

Vue.config.productionTip = false;

new Vue({
  router,
  store,
  render: (h) => h(App),
}).$mount("#app");
