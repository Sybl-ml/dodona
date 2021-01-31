<template>
  <b-container fluid>
    <b-row>
      <b-col lg="8" v-if="ready">
        <h4>Description:</h4>
        <p>{{ description }}</p>
        <h4>Linked Dataset:</h4>
        <b-button-group size="sm" class="mb-3">
          <b-button variant="secondary" @click="$emit('input-tab')">{{ datasetName }}</b-button>
          <b-button variant="outline-secondary" v-b-modal.deleteDataCheck
            >X</b-button
          >
        </b-button-group>

        <b-modal
          id="deleteDataCheck"
          ref="deleteDataCheck"
          title="Are your sure?"
          hide-footer
        >
          <p>You are removing {{ datasetName }} from this project</p>
          <p>Please confirm you are happy to continue</p>
          <b-row class="justify-content-center text-center">
            <b-button class="m-2" variant="success" @click="deleteDataset"
              >Confirm</b-button
            >
            <b-button
              class="m-2"
              variant="warning"
              @click="$bvModal.hide('deleteDataCheck')"
              >Cancel</b-button
            >
          </b-row>
        </b-modal>

        <h4>Job Configuration:</h4>
        <b-progress
          :value="value"
          :variant="progressColor"
          height="2rem"
          show-progress
          animated
        ></b-progress>
        <br />
        <h3>Almost Done!</h3>
        <p>To start computation click the button below</p>
        <br />
        <b-dropdown
          id="dropdown-form"
          text="Job Configuration"
          block
          variant="outline-primary"
          ref="dropdown"
          class="m-2"
          menu-class="w-100"
        >
          <b-dropdown-text
            >This is where you configure how the job should be
            run</b-dropdown-text
          >
          <b-dropdown-form>
            <b-form-group
              label="Timeout (mins)"
              label-for="dropdown-form-timeout"
            >
              <b-form-input
                id="dropdown-form-timeout"
                size="sm"
                type="number"
                v-model="timeout"
              ></b-form-input>
            </b-form-group>
          </b-dropdown-form>
        </b-dropdown>
        <br />
        <p class="display-1 text-center">
          <b-link @click="start">
            <b-icon-play-fill variant="success" />
          </b-link>
        </p>
      </b-col>

      <b-col lg="8" v-else>
        <h4>Description:</h4>
        <p>{{ description }}</p>
        <h5>To continue you must provide a dataset</h5>
        <b-form-file
          class="mb-3"
          placeholder="Select a dataset"
          drop-placeholder="Drop file here..."
          v-model="file"
        />
        <b-button variant="secondary" @click="addData">Upload</b-button>
      </b-col>
      <!-- <b-col xs="12" lg="4">
        <b-card class="mb-2 shadow-sm">
          <h3>Data Analysis</h3>
          <p>Number of Entires: 1900</p>
          <p>Number of Attributes: 12</p>
          <img
            style="width:100%"
            src="https://www.conceptdraw.com/How-To-Guide/picture/Vertical-bar-chart-Global-competitiveness-index-infrastructure-score.png"
          />
          <img
            style="width:100%"
            src="https://www.conceptdraw.com/How-To-Guide/picture/Vertical-bar-chart-Global-competitiveness-index-infrastructure-score.png"
          />
          <p>INSERT MORE WEKA-LIKE STUFF</p>
        </b-card>
      </b-col> -->
    </b-row>
  </b-container>
</template>

<style scoped>
.input-table {
  overflow-y: scroll;
}
</style>

<script>

export default {
  name: "ProjectOverview",
  data() {
    return {
      timeout: 10,
      value: 64,
      file: null,
    };
  },
  props: {
    projectId: String,
    description: String,
    datasetName: String,
    dataDate: Date,
    dataHead: Object,
    dataTypes: Object,
    ready: Boolean,
  },
  computed: {
    getDatasetDate() {
      return `${this.dataDate.toLocaleString("en-GB", {
        dateStyle: "short",
      })} - ${this.dataDate.toLocaleString("en-GB", {
        timeStyle: "short",
      })}`;
    },
    progressColor() {
      if (this.value === 100) {
        return "completed";
      } else if (this.value < 25) {
        return "warning";
      } else if (this.value < 50) {
        return "primary";
      } else if (this.ready < 75) {
        return "ready";
      }
    },
  },
  methods: {
    async start() {
      let user_id = $cookies.get("token");
      try {
        await this.$http.post(
          `http://localhost:3001/api/projects/p/${this.projectId}/process`,
          {
            timeout: this.timeout,
          }
        );
      } catch (err) {
        console.log(err);
      }

      // this.$router.replace("/dashboard/"+this.projectId);
      this.$emit("update:project", this.projectId);
    },
    async deleteDataset() {
      try {
        let project_response = await this.$http.delete(
          `http://localhost:3001/api/projects/p/${this.projectId}/data`
        );
      } catch (err) {
        console.log(err);
      }
      this.$refs["deleteDataCheck"].hide();
    },
    addData() {
      if (this.file) {
        this.file_reader = new FileReader();
        this.file_reader.onload = this.sendFile;
        this.file_reader.readAsText(this.file);
      }
    },
    async sendFile(e) {
      let project_response = await this.$http.put(
        `http://localhost:3001/api/projects/p/${this.projectId}/data`,
        {
          name: this.file.name,
          content: e.target.result,
        }
      );

      // On Success should update dashboard using emitters
    },
  },
};
</script>
