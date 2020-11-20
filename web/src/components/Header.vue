<template>
  <b-navbar :key="$route.fullPath">
    <b-navbar-brand :to="logoRoute">
      <icon-logo width="5em" height="3em" :show_text="true" />
    </b-navbar-brand>
    <b-navbar-nav v-if="atDashboard">
      <b-nav-item disabled> {{ time }} </b-nav-item>
    </b-navbar-nav>
    <b-navbar-nav v-else-if="atLanding">
      <b-nav-item>Product</b-nav-item>
      <b-nav-item>Meet the Team</b-nav-item>
      <b-nav-item>Pricing</b-nav-item>
    </b-navbar-nav>

    <b-navbar-nav class="ml-auto" v-if="loggedIn">
      <b-nav-item disabled>{{credits}} Credits</b-nav-item>
      <b-nav-item-dropdown right>
        <template #button-content>
          <img :src="avatar" class="img-circle">
          {{name}}
        </template>
        <b-dropdown-item disabled>{{ email }}</b-dropdown-item>
        <b-dropdown-divider />
        <b-dropdown-item to="/dashboard">Dashboard</b-dropdown-item>
        <b-dropdown-item to="/client/confirm">{{
          client ? "Nodes" : "Register As Client"
        }}</b-dropdown-item>
        <b-dropdown-divider />
        <b-dropdown-item to="/settings">My Profile</b-dropdown-item>
        <b-dropdown-item to="#">Help</b-dropdown-item>
        <b-dropdown-divider />
        <b-dropdown-item @click="signout">Sign Out</b-dropdown-item>
      </b-nav-item-dropdown>
    </b-navbar-nav>

    <b-navbar-nav v-else class="ml-auto">
      <b-nav-form>
        <b-nav-item><router-link to="/login">Sign In</router-link></b-nav-item>
        <b-button variant="primary" to="/register">SIGN UP NOW</b-button>
      </b-nav-form>
    </b-navbar-nav>
  </b-navbar>
</template>

<style>
.img-circle {
  height: 2rem;
  border-radius: 50%;
}
</style>

<script>
import IconBase from "./IconBase";
import IconLogo from "./icons/IconLogo";
import axios from "axios";

export default {
  name: "Header",
  components: {
    IconLogo,
  },
  data() {
    let randNo = Math.floor(Math.random() * 6) + 1; 
    return {
      name: "",
      email: "",
      client: false,
      time: "",
      credits: 0,
      avatar: "https://www.w3schools.com/w3images/avatar" + randNo + ".png",
      loggedIn: false,
      logoRoute: "/",
      atDashboard: false,
      atLanding: false,
    };
  },
  methods: {
    signout: function () {
      $cookies.remove("token");
      this.$router.push("/login");
    },
    getUserData: async function () {
      let user_id = $cookies.get("token");
      if (!user_id) {
        return;
      }
      try {
        let user_data = await axios.get(
          `http://localhost:3001/api/users/${user_id}`
        );
        this.name = user_data.data.first_name + " " + user_data.data.last_name;
        this.email = user_data.data.email;
        this.client = user_data.client;
        this.credits = user_data.data.credits;
      } catch (err) {
        console.log(err);
      }
    },
    updateHeader: function () {
      let user_id = $cookies.get("token");

      this.getUserData();

      this.loggedIn = user_id ? true : false;
      this.logoRoute = user_id ? "/dashboard" : "/";

      let pageName = this.$route.name;

      this.atLanding = pageName == "Welcome" ? true : false;
      this.atDashboard =
        pageName === "Dashboard" || pageName === "Settings" ? true : false;
    },
  },
  async mounted() {
    this.getUserData();

    setInterval(() => {
      this.time = new Date().toLocaleString("en-GB", {
        dateStyle: "long",
        timeStyle: "medium",
      });
      this.time = this.time.toString().replace(" at", ",");
    }, 1000);
  },
  created() {
    this.updateHeader();
  },
  watch: {
    $route: function () {
      this.updateHeader();
    },
  },
};
</script>
