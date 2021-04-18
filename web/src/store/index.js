import Vue from "vue";
import Vuex from "vuex";
import createPersistedState from "vuex-persistedstate";
import $http from "../services/axios-instance";
import _ from "lodash";
import Papa from "papaparse";
import router from "../router";

Vue.use(Vuex);

function unpackProjectResponse(response) {
  let {
    project,
    details = {
      dataset_date: null,
      dataset_name: "",
      dataset_head: {},
      dataset_types: {},
      train_size: 0,
      predict_size: 0,
    },
    analysis = {},
  } = response;

  if ("head" in details) {
    details.dataset_head = Papa.parse(details.head, { header: true });
    details.dataset_date = new Date(details.date_created.$date);

    delete details.head;
    delete details.date_created;
  }
  project = _.assign(project, {
    _id: project._id.$oid,
    date_created: new Date(project.date_created.$date),
  });
  project.details = details;
  project.analysis = analysis;

  return project;
}

export default new Vuex.Store({
  plugins: [createPersistedState()],
  state: {
    projects: [],
    models: [],
    user_data: {},
  },
  getters: {
    filteredProjects: (state) => (search) => {
      return state.projects.filter((x) => {
        if (x.name.includes(search)) {
          return x;
        }

        if (x.tags.includes(search)) {
          return x;
        }
      });
    },
    getProject: (state) => (id) => {
      let p = state.projects.filter((project) => project._id == id);
      return p[0];
    },
    isAuthenticated: (state) => {
      return !_.isEmpty(state.user_data);
    },
    getModelPerformance: (state) => (id) => {
      let index = state.models.findIndex((m) => m._id.$oid == id);
      let performance = state.models[index].performance;
      return performance;
    },
  },
  mutations: {
    setProjects(state, projects) {
      state.projects = projects;
    },
    setModels(state, models) {
      state.models = models;
    },
    setModelPerformance(state, { performance, id }) {
      let index = state.models.findIndex((m) => m._id.$oid == id);
      Vue.set(state.models[index], "performance", performance);
    },
    setUser(state, user) {
      Vue.set(state, "user_data", user);
    },
    setAvatar(state, avatar) {
      Vue.set(state.user_data, "avatar", avatar);
    },
    addProject(state, new_project) {
      let index = 0;
      if (
        (index = state.projects.findIndex(
          (project) => project._id == new_project._id
        )) !== -1
      ) {
        Vue.set(state.projects, index, new_project);
      } else {
        state.projects.push(new_project);
      }
    },
    updateProject(state, { project_id, field, new_data }) {
      let project = state.projects.find((p) => p._id == project_id);
      Vue.set(project, field, new_data);
    },
    deleteProject(state, id) {
      let index = state.projects.findIndex((p) => p._id == id);
      state.projects.splice(index, 1);

      let new_route = "/";
      if (index >= 1) {
        new_route += state.projects[index - 1]._id;
      } else if (index == 0 && state.projects.length > 0) {
        new_route += state.projects[index]._id;
      } else {
        new_route = "";
      }
      router.replace(`/dashboard${new_route}`);
    },
    unlockModel(state, model_id) {
      let index = state.models.findIndex((m) => m._id.$oid == model_id);
      Vue.set(state.models[index], "locked", false);
    },
  },
  actions: {
    async getProjects({ commit }) {
      let response = await $http.get(`api/projects`);

      let project_response = response.data.map((x) => {
        let p = unpackProjectResponse(x);
        return p;
      });

      commit("setProjects", project_response);

      if (project_response.length > 0) {
        if (!("projectId" in router.currentRoute.params)) {
          router.replace({
            name: `ProjectView`,
            params: {
              projectId: project_response[0]._id,
            },
          });
        }
      }
      console.log("Fetched projects");
    },
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
    async addProject(context, id) {
      let project_response = await $http.get(`api/projects/${id}`);

      let new_project = unpackProjectResponse(project_response.data);
      context.commit("addProject", new_project);
    },
    async postNewProject(context, { name, description, tags }) {
      return $http.post(`api/projects/new`, {
        name: name,
        description: description,
        tags: tags,
      });
    },
    async sendFile(context, { project_id, multipart, files }) {
      let formData = new FormData();
      let config = { headers: { "Content-Type": "multipart/form-data" } };

      let route = "";
      if (!multipart) {
        formData.append("dataset", files);
        route = "upload_and_split";
      } else {
        formData.append(files.train);
        formData.append(files.predict);
        route = "upload_train_and_predict";
      }
      try {
        let response = await $http.put(
          `api/projects/${project_id}/${route}`,
          formData,
          config
        );

        let updated_project = unpackProjectResponse(response.data);
        context.commit("addProject", updated_project);
      } catch (error) {
        console.error(error);
      }
    },
    async startProcessing(
      context,
      {
        projectId,
        node_computation_time,
        cluster_size,
        prediction_type,
        prediction_column,
      }
    ) {
      node_computation_time = parseInt(node_computation_time);
      cluster_size = parseInt(cluster_size);

      let payload = {
        nodeComputationTime: node_computation_time,
        clusterSize: cluster_size,
        predictionType: prediction_type,
        predictionColumn: prediction_column,
      };

      try {
        await $http.post(`api/projects/${projectId}/process`, payload);
      } catch (err) {
        console.log(err);
      }

      context.commit("updateProject", {
        project_id: projectId,
        field: "status",
        new_data: "Processing",
      });
    },
    async updateProject(context, { field, new_data, project_id }) {
      let payload = {
        changes: {
          [field]: new_data,
        },
      };

      try {
        await $http.patch(`api/projects/${project_id}`, payload);
      } catch (err) {
        console.log(err);
        return;
      }

      context.commit("updateProject", {
        project_id: project_id,
        field: field,
        new_data: new_data,
      });
    },
    async deleteProject({ commit }, { projectId }) {
      try {
        await $http.delete(`api/projects/${projectId}`);
      } catch (err) {
        console.log(err);
        return;
      }

      commit("deleteProject", projectId);
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
    async generatePrivateKey({ commit }) {
      console.log("Generating new private key");
      return $http.post("api/clients/generatePrivateKey");
    },
    async uploadAvatar(context, avatar) {
      return $http.post("api/users/avatar", {
        avatar,
      });
    },
    async unlockModel(context, { model_id, password }) {
      try {
        let response = await $http.post(
          `api/clients/models/${model_id}/unlock`,
          {
            password: password,
          }
        );
        console.log(`Unlocking Model ${model_id}`);
        context.commit("unlockModel", model_id);
      } catch (err) {
        console.log(err);
      }
    },
    async deleteData(context, projectId) {
      try {
        await $http.delete(`api/projects/${projectId}/data`);
      } catch (err) {
        console.log(err);
      }
      context.commit("updateProject", {
        project_id: projectId,
        field: "status",
        new_data: "Unfinished",
      });
    },
  },
});
