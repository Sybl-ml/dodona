<template>
  <div>
    <Header />
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
                    projectId: p.id,
                    name: p.name,
                    description: p.description,
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
  </div>
</template>

<style>
.add-new {
  height: 40px;
  font-size: large;
}
</style>
<script>
import Header from "@/components/headers/Dashboard";
import axios from "axios";

export default {
  name: "Dashboard",
  components: {
    Header,
  },
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
