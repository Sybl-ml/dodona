import $http from "../../services/axios-instance";
import router from "../../router";
import Vue from "vue";

const state = () => ({
  user_data: {},
});

const getters = {
  isAuthenticated: (state) => {
    return !_.isEmpty(state.user_data);
  },
};

const mutations = {
  setUser(state, user) {
    Vue.set(state, "user_data", user);
  },
  updateClientStatus(state) {
    Vue.set(state.user_data, "client", true);
  },
  setAvatar(state, avatar) {
    Vue.set(state.user_data, "avatar", avatar);
  },
};

const actions = {
  async getUserData(context) {
    if (context.user_data) {
      return;
    }
    return $http.get(`api/users`);
  },
  async getAvatar({ commit }) {
    let response = await $http.get(`api/users/avatar`);
    commit("setAvatar", response.data.img);
  },
  async postNewAvatar(context, avatar) {
    try {
      await $http.post("api/users/avatar", {
        avatar: avatar,
      });
    } catch (err) {
      console.log(err);
    }

    context.commit("setAvatar", avatar);
  },
  async login({ commit }, { email, password }) {
    return $http.post("api/users/login", {
      email: email,
      password: password,
    });
  },
  async logout({ commit }) {
    Vue.prototype.$cookies.remove("token");
    commit("setUser", {});
    router.push("/login");
  },
  async register(
    { commit },
    { email, password, firstName, lastName, currency, dob }
  ) {
    return $http.post("api/users/new", {
      email: email,
      password: password,
      firstName: firstName,
      lastName: lastName,
      currency: currency,
      dob: dob,
    });
  },
  async client_register({ commit }, { id, email, password }) {
    let response = await $http.post("api/clients/register", {
      id: id,
      email: email,
      password: password,
    });
    commit("updateClientStatus");
    return response;
  },
  async uploadAvatar(context, avatar) {
    return $http.post("api/users/avatar", {
      avatar,
    });
  },
};

export default {
  state,
  getters,
  actions,
  mutations,
};
