import router from "../../router";
import $http from "../../services/axios-instance";
import Vue from "vue";

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
    status: typeof project.status === "object" ? "Processing" : project.status,
    progress:
      typeof project.status === "object" ? project.status.Processing : {},
  });
  project.details = details;
  project.analysis = analysis;

  return project;
}

const state = () => ({
  projects: [],
});

// getters
const getters = {
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
    let p = state.projects.find((project) => project._id == id);
    return p;
  },
  getProjectStatus: (state, getters) => (id) => {
    let p = getters.getProject(id);

    if (typeof p.status === "object") return "Processing";
    else return p.status;
  },
  getProjectProgress: (state, getters) => (id) => {
    let p = getters.getProject(id);

    if ("current_job" in p && !_.isEmpty(p.current_job)) {
      return {
        ...p.progress,
        max: p.current_job.config.cluster_size,
      };
    }
  },
};

// mutations
const mutations = {
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
  startJob(state, { project_id, job }) {
    let project = state.projects.find((p) => p._id == project_id);

    Vue.set(project, "status", "Processing");
    Vue.set(project, "current_job", job);
    Vue.set(project, "progress", { model_success: 0, model_err: 0 });
  },
  addJobToProject(state, { project_id, job }) {
    let project = state.projects.find((p) => p._id == project_id);

    Vue.set(project, "current_job", job.job);
  },
  addJobStatsToProject(state, { project_id, job_stats }) {
    let project = state.projects.find((p) => p._id == project_id);

    Vue.set(project, "job_stats", job_stats);
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
  SOCKET_ONMESSAGE(state, message) {
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

        let p = state.projects.find((project) => project._id == project_id);

        if (success)
          Vue.set(p.progress, "model_success", p.progress.model_success + 1);
        else Vue.set(p.progress, "model_err", p.progress.model_err + 1);
        break;
      default:
        console.err("Unknown Message");
    }
  },
};

// actions
const actions = {
  async getProjects({ dispatch, commit }) {
    let response = await $http.get(`api/projects`);

    let project_response = response.data.map((x) => {
      let p = unpackProjectResponse(x);
      return p;
    });

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
    commit("setProjects", project_response);

    for (let project of project_response) {
      if (project.status === "Processing") {
        await dispatch("getRecentJob", project._id);
      } else if (project.status === "Complete") {
        await dispatch("getRecentJob", project._id);
        await dispatch("getJobStatistics", project._id);
      }
    }
    console.log("Fetched projects");
  },
  async getRecentJob({ commit }, id) {
    let job = await $http.get(`api/projects/${id}/job`);

    commit("addJobToProject", {
      project_id: id,
      job: job.data,
    });
  },
  async getJobStatistics({ commit }, project_id) {
    let job_stats = await $http.get(
      `api/projects/${project_id}/job_statistics`
    );

    commit("addJobStatsToProject", {
      project_id: project_id,
      job_stats: job_stats.data,
    });
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
  async sendFile(context, { project_id, multifile, files }) {
    let formData = new FormData();
    let config = { headers: { "Content-Type": "multipart/form-data" } };

    context.commit("updateProject", {
      project_id: project_id,
      field: "status",
      new_data: "Uploading",
    });

    let route = "";
    if (!multifile) {
      formData.append("dataset", files);
      route = "upload_and_split";
    } else {
      formData.append("train", files.train);
      formData.append("predict", files.predict);
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
      let response = await $http.post(
        `api/projects/${projectId}/process`,
        payload
      );

      context.commit("startJob", {
        project_id: projectId,
        job: response.data,
      });
    } catch (err) {
      console.log(err);
    }
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
};

export default {
  state,
  getters,
  actions,
  mutations,
};
