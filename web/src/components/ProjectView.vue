<template>
  <b-container fluid>
    <h2>{{ name }}</h2>
    <h5>{{ description }}</h5>
    <p>{{ getProjectDate }}</p>
    <b-card no-body class="shadow">
    <b-tabs pills card>
      <b-tab title="Overview" active lazy ref="overviewTab">
        <project-overview
          :projectId="projectId"
          :key="projectId"
          :dataHead="dataHead"
          :dataDate="datasetDate"
          :dataTypes="dataTypes"
          :ready="status=='Ready'"
          @update:project="updateProject"
          v-on:input-tab="viewInput"
        />
      </b-tab>
      <b-tab title="Input" ref="inputTab">
        <project-input
          :projectId="projectId"
          :key="projectId"
          @get-data="fetchData"
          :data="data"
          :loading="loading"
        />
      </b-tab>
      <b-tab title="Ouptut" lazy>
        <project-output
          :projectId="projectId"
          :key="projectId"
          @get-results="fetchResults"
          :results="results"
          :loading="results_loading"
        />
      </b-tab>
      <b-tab title="Settings" lazy>
        <project-settings
          :projectId="projectId"
          :key="projectId"
          :name="name"
          :description="description"
          @update:name="updateName"
          @update:description="updateDescription"
          @delete:project="$emit('delete:project', projectId)"
        />
      </b-tab>
    </b-tabs>
    </b-card>
  </b-container>
</template>

<script>
import axios from "axios";
import Papa from "papaparse";
import ProjectOverview from "@/components/ProjectOverview";
import ProjectInput from "@/components/ProjectInput";
import ProjectOutput from "@/components/ProjectOutput";
import ProjectSettings from "@/components/ProjectSettings";

export default {
  name: "ProjectView",
  data() {
    return {
      name: "",
      description: "",
      status: "",
      dateCreated: new Date(),

      datasetDate: new Date(),
      dataHead: {},
      dataTypes: {},

      data: null,
      loading: false,

      results: null,
      results_loading: false,
    };
  },
  props: {
    projectId: String,
  },
  components: {
    ProjectOverview,
    ProjectInput,
    ProjectOutput,
    ProjectSettings,
  },
  watch: {
    projectId: function () {
      this.resetProject();
      this.fetchProject();
      this.$refs.overviewTab.activate();
    },
  },
  async mounted() {
    this.fetchProject();
  },
  methods: {
    async fetchProject() {
      let project_response = await axios.get(
        `http://localhost:3001/api/projects/p/${this.projectId}`
      );

      let project_details = project_response.data.details;
      let project_info = project_response.data.project;
      
      this.name = project_info.name;
      this.description = project_info.description;
      this.dateCreated = new Date(project_info.date_created.$date);
      this.status = project_info.status;
      this.dataHead = Papa.parse(project_details.head, { header: true });
      this.datasetDate = new Date(project_details.date_created.$date);
      this.dataTypes = project_details.column_types;
    },
    async fetchData() {
      this.loading = true;

      let project_response = await axios.get(
        `http://localhost:3001/api/projects/p/${this.projectId}/data`
      );

      let project_data = project_response.data.dataset;

      this.data = Papa.parse(project_data, { header: true });
      this.loading = false;
    },
    async fetchResults() {
      this.results_loading = true;

      let project_predictions = await axios.get(
        `http://localhost:3001/api/projects/p/${this.projectId}/predictions`
      );
      this.results = project_predictions.data['predictions'];
      this.results_loading = false;
    },
    resetProject() {
      this.name = "";
      this.description = "";
      this.dateCreated = new Date();

      this.datasetDate = new Date();
      this.dataHead = {};
      this.dataTypes = {};

      this.data = null;
      this.loading = false;
    },
    viewInput() {
      this.$refs.inputTab.activate();
      this.fetchData();
    },
    updateName(newName) {
      this.name = newName;
      this.$emit("update:name", newName, this.projectId);
    },
    updateDescription(newDescription) {
      this.description = newDescription;
      this.$emit("update:description", newDescription, this.projectId);
    },
    updateProject(id) {
      console.log("hi2")
      this.$emit("update:project", id);
    },
  },
  computed: {
    getProjectDate() {
      return `${this.dateCreated.toLocaleString("en-GB", {
        dateStyle: "short",
      })} - ${this.dateCreated.toLocaleString("en-GB", {
        timeStyle: "short",
      })}`;
    },
  },
};
</script>
