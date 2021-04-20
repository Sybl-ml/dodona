import Vue from "vue";
import Vuex from "vuex";
import createPersistedState from "vuex-persistedstate";
import $http from "../services/axios-instance";
import _ from "lodash";
import Papa from "papaparse";
import router from "../router";

import projects from "./modules/projects";
import models from "./modules/models";
import user_data from "./modules/users";
import socket from "./modules/socket";
Vue.use(Vuex);

export default new Vuex.Store({
  // plugins: [createPersistedState()],
  modules: {
    projects,
    models,
    user_data,
    socket,
  },
  actions: {
    async generatePrivateKey({ commit }) {
      console.log("Generating new private key");
      return $http.post("api/clients/generatePrivateKey");
    },
  },
});
