<template>
  <b-container fluid>
    <h2>{{ name }}</h2>
    <h5>{{ description }}</h5>
    <p>{{ getProjectDate }}</p>
    <b-tabs>
      <b-tab title="Overview" active lazy ref="overviewTab">
        <br />
        <project-overview
          :projectId="projectId"
          :key="projectId"
          :dataHead="dataHead"
          :dataDate="datasetDate"
          :dataTypes="dataTypes"
          v-on:input-tab="viewInput"
        />
      </b-tab>
      <b-tab title="Input" ref="inputTab">
        <br />
        <project-input
          :projectId="projectId"
          :key="projectId"
          @get-data="fetchData"
          :data="data"
          :loading="loading"
        />
      </b-tab>
      <b-tab title="Ouptut" lazy>
        <br />
        This will show the output from the machine learning methods
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
  </b-container>
</template>

<script>
import axios from "axios";
import Papa from "papaparse";
import ProjectOverview from "@/components/ProjectOverview";
import ProjectInput from "@/components/ProjectInput";
import ProjectSettings from "@/components/ProjectSettings";

export default {
  name: "ProjectView",
  data() {
    return {
      name: "",
      description: "",
      dateCreated: new Date(),

      datasetDate: new Date(),
      dataHead: {},
      dataTypes: {},

      data: null,
      loading: false,
    };
  },
  props: {
    projectId: String,
  },
  components: {
    ProjectOverview,
    ProjectInput,
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
