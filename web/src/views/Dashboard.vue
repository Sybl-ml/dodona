<template>
  <b-container fluid>
    <b-row>
      <b-col xs="12" order-xs="2" lg="3">
        <b-row>
          <b-col class="mb-3">
            <b-form-input placeholder="Search" block />
          </b-col>
        </b-row>
        <b-row class="text-left">
          <b-col>
            <router-link :to="{ name: 'AddProject' }">
              <b-button variant="primary" class="mb-1 add-new" block
                ><b-row
                  ><b-col class="text-left">Add new project</b-col
                  ><b-col class="ml-auto text-right">
                    <b-icon-plus-circle /></b-col></b-row></b-button
            ></router-link>
            <router-link
              v-for="p in projects"
              :key="p.id"
              :to="{
                name: `ProjectView`,
                params: {
                  projectId: p._id.$oid,
                  name: p.name,
                  description: p.description,
                  dateCreated: new Date(p.date_created.$date),
                },
              }"
            >
              <b-card :title="p.name" class="mb-1">
                <b-card-text>
                  {{ p.description }}
                </b-card-text>
              </b-card>
            </router-link>
          </b-col>
        </b-row>
      </b-col>
      <b-col>
        <router-view></router-view>
      </b-col>
    </b-row>
  </b-container>
</template>

<style>
.add-new {
  height: 40px;
  font-size: large;
}
</style>
<script>
import axios from "axios";

export default {
  data() {
    return {
      projects: [],
    };
  },
  async mounted() {
    let user_id = $cookies.get("token");

    let response = await axios.get(
      `http://localhost:3001/api/projects/u/${user_id}`
    );

    this.projects = response.data;
  },
  methods: {},
};
</script>
