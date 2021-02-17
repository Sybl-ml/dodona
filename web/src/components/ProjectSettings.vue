<template>
  <b-container fluid class="mt-3">
    <h4>Edit your Project</h4>
    <b-form-group label="Project Name" id="name" class="font-weight-bold">
      <b-row no-gutters class="mb-2">
        <b-col lg="3" class="pr-2"
          ><b-form-input id="name" v-model="newName"></b-form-input
        ></b-col>
        <b-col
          ><b-button variant="primary" @click="updateName"
            >Rename</b-button
          ></b-col
        >
      </b-row>
    </b-form-group>
    <b-form-group
      label="Project Description"
      id="desc"
      class="font-weight-bold"
    >
      <b-form-textarea
        id="desc"
        label="Change Project Description"
        v-model="newDescription"
        class="mb-2"
      />
      <b-button variant="primary" @click="updateDescription"
        ><b-icon-pen /> Edit</b-button
      >
    </b-form-group>
    <b-card border-variant="secondary" class="mt-3 shadow">
      <b-form-group
        label="Delete Project"
        description="WARNING: This is permanent all data and analysis will be deleted"
        class="font-weight-bold"
      >
        <b-button id="delete" variant="secondary"  v-b-modal.deleteCheck
          >DELETE</b-button
        >

        <b-modal id="deleteCheck" ref="deleteCheck" title="Are your sure?" hide-footer>
          <p>You are removing this project: {{name}}</p>
          <p> Please confirm you are happy to continue</p>
          <b-row class="justify-content-center text-center">
            <b-button class="m-2" variant="success" @click="deleteProject">Confirm</b-button>
            <b-button class="m-2" variant="warning" @click="$bvModal.hide('deleteCheck')">Cancel</b-button>
          </b-row>
        </b-modal>
      </b-form-group>
    </b-card>
  </b-container>
</template>

<style scoped>
.danger-zone {
  border: 1px solid red;
  border-radius: 3px;
}
</style>

<script>
export default {
  name: "ProjectSettings",
  data() {
    return {
      newName: this.name,
      newDescription: this.description,
    };
  },
  props: {
    projectId: String,
    name: String,
    description: String,
  },
  methods: {
    async updateName() {
      this.$emit("update:name", this.newName);

      try {
        let project_response = await this.$http.patch(
          `http://localhost:3001/api/projects/p/${this.projectId}`,
          {
            changes: {
              name: this.newName,
            }
          }
        );
      } catch (err) {
        console.log(err);
      }
    },
    async updateDescription() {
      this.$emit("update:description", this.newDescription);

      try {
        let project_response = await this.$http.patch(
          `http://localhost:3001/api/projects/p/${this.projectId}`,
          {
            changes: {
              description: this.newDescription,
            }
          }
        );
      } catch (err) {
        console.log(err);
      }
    },
    async deleteProject() {
      this.$emit("delete:project", this.projectId);

      try {
        let project_response = await this.$http.delete(
          `http://localhost:3001/api/projects/p/${this.projectId}`
        );
      } catch (err) {
        console.log(err);
      }

      this.$refs['deleteCheck'].hide();
      this.$router.replace("/dashboard");
    },
  },
};
</script>
