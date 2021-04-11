import Vue from "vue";
import Vuex from "vuex";
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
  state: {
    projects: [],
    models: [],
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
      console.log(id);
      console.log(state.projects);
      let p = state.projects.filter((project) => project._id == id);
      console.log(p);
      return p[0];
    },
  },
  mutations: {
    setProjects(state, projects) {
      state.projects = projects;
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
      } else if (index == 0 && state.models.length > 0) {
        new_route += state.projects[index]._id;
      } else {
        new_route = "";
      }
      router.replace(`/dashboard${new_route}`);
    },
  },
  actions: {
    async getProjects({ commit }) {
      let response = await $http.get(`api/projects`);

      let project_response = response.data.map((x) => {
        let p = unpackProjectResponse(x);
        return p;
      });
      console.log(project_response);

      commit("setProjects", project_response);
      console.log("Fetched projects");
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
      console.log(payload);
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
  },
});
