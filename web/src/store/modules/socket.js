import Vue from "vue";

const state = () => ({
  socket: {
    isConnected: false,
    authenticated: false,
    message: "",
    reconnectError: false,
  },
});

const getters = {};

const mutations = {
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
  },
};
const actions = {
  sendMsg(context, msg) {
    Vue.prototype.$socket.sendObj(msg);
  },
};

export default {
  state,
  mutations,
  actions,
};
