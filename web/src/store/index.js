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
Vue.use(Vuex);

export default new Vuex.Store({
  // plugins: [createPersistedState()],
  modules: {
    projects,
    models,
    user_data,
    // socket,
  },
  state: {
    // projects: [],
    // models: [],
    // user_data: {},
    socket: {
      isConnected: false,
      authenticated: false,
      message: "",
      reconnectError: false,
    },
  },
  mutations: {
    SOCKET_ONOPEN(state, event) {
      Vue.prototype.$socket = event.currentTarget;
      state.socket.isConnected = true;

      let token = Vue.prototype.$cookies.get("token");
      let auth = {
        authentication: { token: token },
      };
      console.log("sending auth msg");
      Vue.prototype.$socket.sendObj(auth);
    },
    SOCKET_ONCLOSE(state, event) {
      state.socket.isConnected = false;
      state.socket.authenticated = false;
    },
    SOCKET_ONMESSAGE(state, message) {
      state.socket.message = message;

      console.log(message);
      switch (Object.keys(message)[0]) {
        case "hello":
          break;
        case "modelComplete":
          let {
            project_id,
            cluster_size,
            model_complete_count,
            success,
          } = message.modelComplete;

          projects.commit("updateProjectProgress", {
            project_id: project_id,
            success: success,
          });

          break;
        default:
          console.err("Unknown Message");
      }
    },
  },
  actions: {
    sendMsg(context, msg) {
      Vue.prototype.$socket.sendObj(msg);
    },
    async generatePrivateKey({ commit }) {
      console.log("Generating new private key");
      return $http.post("api/clients/generatePrivateKey");
    },
  },
});
