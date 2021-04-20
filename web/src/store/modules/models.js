import router from "../../router";
import $http from "../../services/axios-instance";

const state = () => ({
  models: [],
});

const getters = {
  getModelPerformance: (state) => (id) => {
    let model = state.models.find((m) => m._id.$oid == id);
    let performance = model.performance;
    return performance;
  },
};

const mutations = {
  setModels(state, models) {
    state.models = models;
  },
  setModelPerformance(state, { performance, id }) {
    let model = state.models.find((m) => m._id.$oid == id);
    Vue.set(model, "performance", performance);
  },
  unlockModel(state, model_id) {
    let index = state.models.findIndex((m) => m._id.$oid == model_id);
    Vue.set(state.models[index], "locked", false);
  },
};

const actions = {
  async getModels({ commit }) {
    try {
      let data = await $http.get(`api/clients/models`);

      commit("setModels", data.data);
    } catch (err) {
      console.log(err);
    }
  },
  async getModelPerformance(context, id) {
    try {
      let data = await $http.get(`api/clients/models/${id}/performance`);
      context.commit("setModelPerformance", {
        performance: data.data,
        id: id,
      });
    } catch (err) {
      console.log(err);
    }
  },
  async unlockModel(context, { model_id, password }) {
    try {
      await $http.post(`api/clients/models/${model_id}/unlock`, {
        password: password,
      });
      console.log(`Unlocking Model ${model_id}`);
      context.commit("unlockModel", model_id);
    } catch (err) {
      console.log(err);
    }
  },
};

export default {
  state,
  getters,
  actions,
  mutations,
};
